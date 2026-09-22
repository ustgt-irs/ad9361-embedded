use super::{DelayNs, OutputPin, SpiDevice, bisync};
use arbitrary_int::{traits::Integer as _, u2, u3, u4, u5, u6, u7, u10, u13};

use crate::types::{
    FastAgcTargetGainIndexType, GainControlMode, ReceiverId, RxPhaseConfig, TransmitterId,
};
use crate::{clocks, config, lut, regs, spi};

use crate::{
    clocks::{AdcDivisor, Clocks},
    config::{
        AuxAdcConfig, DivisionDuplexConfig, DivisionDuplexMode, ExternalLnaConfig, GainControl,
        RssiConfig, TransceiverChannelMode,
    },
    limits::RSSI_MAX_WEIGHT,
    lut::{LutFrequency, RxGainTable, RxGainTableType, SynthLutType},
};

#[cfg(feature = "axi-tune")]
use axi_ad9361::regs::{
    adc::fields::{ChannelDataPathControl, ChannelStatus, PnSel},
    dac::regs::{ChannelDataSource, ChannelLegacyControl, DataSource},
};

/// Product ID and silicon revision, as read back from the AD9361. See [`Ad9361::read_product_id`]/
/// [`Ad9361Uninit::read_product_id`].
#[bitbybit::bitfield(u8, default = 0x0, debug)]
pub struct ProductIdReg {
    /// Product ID.
    #[bits(3..=7, rw)]
    product_id: u5,
    /// Silicon revision.
    #[bits(0..=2, rw)]
    rev: u3,
}

/// Returned by [`Ad9361::mute_tx`]. Holds the TX1/TX2 attenuation that was active before muting.
///
/// Restoring it requires an async SPI write, which [`Drop`] cannot perform — so this type must
/// be explicitly consumed via [`Self::unmute`]. A guard dropped without calling `unmute()` is a
/// bug: TX stays muted, and `Drop` asserts on it in debug builds since there is nothing else it
/// can do.
#[must_use = "dropping this without calling `unmute` leaves TX muted"]
pub struct TxMuteGuard {
    tx1_md_b: u32,
    tx2_md_b: u32,
    restored: bool,
}

#[bisync]
impl TxMuteGuard {
    /// Restores the TX1/TX2 attenuation that was active before [`Ad9361::mute_tx`] was called.
    pub async fn unmute<Spi: SpiDevice, ResetPin: OutputPin, Delay: DelayNs>(
        mut self,
        ad9361: &mut Ad9361<Spi, ResetPin, Delay>,
    ) -> Result<(), Spi::Error> {
        self.restored = true;
        ad9361.tx_unmute(self.tx1_md_b, self.tx2_md_b).await
    }
}

impl Drop for TxMuteGuard {
    fn drop(&mut self) {
        debug_assert!(
            self.restored,
            "TxMuteGuard dropped without calling unmute() -- TX left muted"
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct GainTableIndexForTxQuad(Option<u7>);

#[derive(Debug, Clone, Copy)]
struct TxQuadCalibParams {
    phase_config: RxPhaseConfig,
    gain_table_index: GainTableIndexForTxQuad,
    phase_inversion_en: bool,
}

/// Raw register values with the tracking enable bits, see [`Ad9361::disable_tracking`].
struct SavedTracking {
    dc_offset_config_2: u8,
    calibration_config_1: u8,
}

#[derive(Debug, Clone)]
struct Ad9361Config {
    // Not read yet. Needed once the ENSM handling supports waking up from sleep.
    #[allow(dead_code)]
    external_clock_config: config::ExternalClockConfig,
    /// Number of AXI ADC/DAC channels (2 for 1R1T, 4 for 2R2T), derived once from
    /// `config.transceiver_channel_mode` at setup. Used by [`Ad9361::digital_tune`] so callers
    /// don't have to pass it in — it can't change without re-running `init`.
    #[cfg(feature = "axi-tune")]
    num_axi_channels: u4,

    // Needed by `Ad9361::update_rf_clocks` to redo the sample rate dependent setup steps.
    // `clock` is replaced on every update, the rest stays as it was during setup.
    reference_clk_rate: u32,
    clock: clocks::ClockConfig,
    rf_rx_bandwidth_hz: u32,
    rf_tx_bandwidth_hz: u32,
    gain_control: config::GainControl,
    elna: Option<config::ExternalLnaConfig>,
    aux_adc: config::AuxAdcConfig,
    rssi: config::RssiConfig,
    transceiver_channel_mode: TransceiverChannelMode,
    /// [`None`] if the TX quad calibration is disabled.
    tx_quad: Option<TxQuadCalibParams>,
    /// Number of taps loaded by [`Ad9361::set_tx_fir_config`], [`None`] before that.
    tx_fir_taps: Option<u8>,
    /// Number of taps loaded by [`Ad9361::set_rx_fir_config`], [`None`] before that.
    rx_fir_taps: Option<u8>,
}

impl Ad9361Config {
    /// Checks the number of TX and RX FIR taps against the limits of `clock`. The limits do not
    /// apply while a FIR is bypassed.
    fn check_fir_taps<E>(
        &self,
        clock: &clocks::ClockConfig,
        tx_taps: Option<u8>,
        rx_taps: Option<u8>,
    ) -> Result<(), InitError<E>> {
        if let Some(taps) = tx_taps
            && let Some(max) = clock.max_tx_fir_taps(self.reference_clk_rate)
            && u32::from(taps) > max
        {
            return Err(InitError::TxFirTapsExceedClockRatio {
                taps: taps.into(),
                max,
            });
        }
        if let Some(taps) = rx_taps
            && let Some(max) = clock.max_rx_fir_taps(self.reference_clk_rate)
            && u32::from(taps) > max
        {
            return Err(InitError::RxFirTapsExceedClockRatio {
                taps: taps.into(),
                max,
            });
        }
        Ok(())
    }

    /// Combines the sample rate dependent `path` with the parts of the clock config that
    /// do not change after setup.
    fn clock_for_path(&self, path: &clocks::BbClockPathConfigHelper) -> clocks::ClockConfig {
        let ref_clk_scalers = self.clock.ref_clk_scalers;
        let bb_pll_ref_clock =
            ref_clk_scalers.calculate_bb_pll_synth_clock(self.reference_clk_rate);
        clocks::ClockConfig {
            ref_clk_scalers,
            bb_pll: clocks::BbPllConfig::calculate(bb_pll_ref_clock, path.bb_pll_clock_hz()),
            adc: path.adc_div(),
            rx: path.rx_config(self.clock.rx.lo),
            tx: path.tx_config(self.clock.tx.lo),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Ad9361State {
    current_ensm_state: EnsmState,
    ensm_pin_control: bool,
}

/// A driver instance before [`Ad9361Uninit::init`] has run. Create one with
/// [`Ad9361Uninit::new`], then call [`Ad9361Uninit::reset`] followed by [`Ad9361Uninit::init`]
/// to get a usable [`Ad9361`].
pub struct Ad9361Uninit<Spi: SpiDevice, ResetPin: OutputPin, Delay: DelayNs>(
    Ad9361Core<Spi, ResetPin, Delay>,
);

/// A driver instance that has completed [`Ad9361Uninit::init`] and is ready for normal use.
pub struct Ad9361<Spi: SpiDevice, ResetPin: OutputPin, Delay: DelayNs> {
    inner: Ad9361Core<Spi, ResetPin, Delay>,
    config: Ad9361Config,
    state: Ad9361State,
}

/// Error from [`Ad9361Uninit::reset`].
#[derive(Debug, thiserror::Error)]
pub enum ResetError<Spi, Gpio> {
    /// SPI error.
    #[error("SPI error: {0}")]
    Spi(Spi),
    /// GPIO error.
    #[error("GPIO error: {0}")]
    Gpio(Gpio),
}

/// Identifies one of the two auxiliary DACs (not the main TX signal DAC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Dac {
    /// First auxiliary DAC.
    Dac1 = 0,
    /// Second auxiliary DAC.
    Dac2 = 1,
}

/// A calibration or setup module, used in [`InitError::CalibrationTimeout`] to say which one
/// timed out.
#[derive(Debug)]
pub enum ModuleId {
    /// BB PLL.
    BbPll,
    /// TX RF PLL.
    TxPll,
    /// RX RF PLL.
    RxPll,
    /// TX quadrature calibration.
    TxQuad,
    /// RF DC offset calibration.
    RfDcOffset,
    /// RX baseband filter tuning.
    RxBbTune,
    /// TX baseband filter tuning.
    TxBbTune,
    /// Baseband DC offset calibration.
    BbDc,
}

/// Error from [`Ad9361Uninit::init`], [`Ad9361::update_rf_clocks`] and the FIR setters of
/// [`Ad9361`].
#[derive(Debug, thiserror::Error)]
pub enum InitError<Spi> {
    /// SPI error.
    #[error("SPI error: {0}")]
    Spi(#[from] Spi),
    /// ENSM error.
    #[error("ENSM error: {0}")]
    Ensm(#[from] EnsmError<Spi>),
    /// The calibration of the module timed out.
    #[error("calibration timeout inside {0:?} module")]
    CalibrationTimeout(ModuleId),
    /// The state register holds an invalid ENSM state.
    #[error("read invalid ENSM state")]
    InvalidEnsmState,
    /// The RX FIR input clock must be equal to or twice the TX FIR output clock.
    #[error(
        "unsymetric clock configuration, RX FIR input must be equal or twice of the TX FIR output"
    )]
    UnsymetricFirClocks {
        /// TX FIR output clock in Hz.
        tx_fir_output_hz: u32,
        /// RX FIR input clock in Hz.
        rx_fir_input_hz: u32,
    },
    /// No gain table index was found for the TX quadrature calibration.
    #[error("could not determine gain table index for TX Quad calibration")]
    NoGainTableIndexForTxQuadCalibrationFound,
    /// The TX FIR has more taps than the clock ratio allows.
    #[error("{taps} TX FIR taps exceed the maximum of {max} for the clock ratio")]
    TxFirTapsExceedClockRatio {
        /// Number of taps in the configuration.
        taps: u32,
        /// Maximum number of taps for the clock ratio.
        max: u32,
    },
    /// The RX FIR has more taps than the clock ratio allows.
    #[error("{taps} RX FIR taps exceed the maximum of {max} for the clock ratio")]
    RxFirTapsExceedClockRatio {
        /// Number of taps in the configuration.
        taps: u32,
        /// Maximum number of taps for the clock ratio.
        max: u32,
    },
}

/// Auxiliary DAC output values in millivolts: `.0` is [`Dac::Dac1`], `.1` is [`Dac::Dac2`].
pub struct AuxDacValuesMv(pub Option<u16>, pub Option<u16>);

/// The AD9361's ENSM (Enable State Machine) state. `DeviceState` wraps the value read from the
/// `State` register; `Sleep` is a fully-powered-down state that isn't reflected in that register
/// (distinct from `regs::EnsmState::SleepWait`, a transitional state on the way in or out of it).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EnsmState {
    /// State reported by the state register.
    DeviceState(regs::EnsmState),
    /// Fully powered down, which the state register does not report.
    Sleep,
}

impl From<regs::EnsmState> for EnsmState {
    fn from(value: regs::EnsmState) -> Self {
        Self::DeviceState(value)
    }
}

/// Result of [`Ad9361Uninit::init`]: the ready-to-use driver plus the clock configuration
/// derived during init, or the error that stopped init partway through.
// The `Spi: SpiDevice` bound isn't enforced here (a type-alias limitation), but it's still
// needed for `Spi::Error` to resolve, and the real enforcement lives on the `impl` block below.
#[allow(type_alias_bounds)]
type Ad9361InitResult<Spi: SpiDevice, ResetPin, Delay> =
    Result<(Ad9361<Spi, ResetPin, Delay>, Clocks), InitError<Spi::Error>>;

#[bisync]
impl<Spi: SpiDevice, ResetPin: OutputPin, Delay: DelayNs> Ad9361Uninit<Spi, ResetPin, Delay> {
    /// Creates a new, unconfigured driver. Call [`Self::reset`] and then [`Self::init`] before
    /// using it.
    pub fn new(spi: Spi, reset_pin: ResetPin, delay: Delay) -> Self {
        Self(Ad9361Core::new(spi, reset_pin, delay))
    }

    /// Hardware-resets the AD9361 via the reset GPIO pin.
    pub async fn reset(&mut self) -> Result<(), ResetError<Spi::Error, ResetPin::Error>> {
        self.0.reset().await
    }

    /// Reads the product ID and silicon revision registers.
    pub async fn read_product_id(&mut self) -> Result<ProductIdReg, Spi::Error> {
        self.0.read_product_id().await
    }

    /// Writes a single register by its raw 10-bit SPI address, bypassing the [`regs::Register`]
    /// enum.
    pub async fn write_register_raw(&mut self, reg: u10, val: u8) -> Result<(), Spi::Error> {
        self.0.write_register_raw(reg, val).await
    }

    /// Reads a single register by its raw 10-bit SPI address, bypassing the [`regs::Register`]
    /// enum.
    pub async fn read_register_raw(&mut self, reg: u10) -> Result<u8, Spi::Error> {
        self.0.read_register_raw(reg).await
    }

    /// Brings the AD9361 up and runs calibration.
    ///
    /// If `config.clock.rx.rx_fir`/`config.clock.tx.tx_fir` request a non-bypass ratio, this call
    /// alone is not enough to get real filtering. The FIR stage is intentionally kept bypassed
    /// through calibration, because running it against not-yet-loaded coefficients would corrupt
    /// calibration. It only switches to the target ratio afterward, using whatever's currently in
    /// the FIR's coefficient RAM. To actually load filter taps, follow this call with
    /// [`Ad9361::set_tx_fir_config`] and [`Ad9361::set_rx_fir_config`]. Skipping that follow-up
    /// call doesn't produce an error. It just silently runs the FIR with whatever coefficients
    /// happen to already be present, which is typically all zero after a fresh power-on reset.
    ///
    /// The C driver also tunes the digital interface during its setup. That needs the AXI ADC
    /// and DAC, so run [`Ad9361::digital_tune`] after this call.
    pub async fn init(
        mut self,
        config: &config::ConfigValidated,
    ) -> Ad9361InitResult<Spi, ResetPin, Delay> {
        let config = &config.0;

        let _actual_mv_values = self.setup_auxdac(&config.aux_dac).await?;
        self.setup_gpo(&config.gpo_config).await?;

        // Needs to be set to 1.
        self.0.write_register(regs::Register::Ctrl, 1).await?;
        // Register map says we should set those to 0x0E, C lib does the same.
        self.0
            .write_register(regs::Register::BandgapConfig0, 0x0E)
            .await?;
        self.0
            .write_register(regs::Register::BandgapConfig1, 0x0E)
            .await?;

        if let config::ExternalClockConfig::Dcxo {
            coarse_tune: dcxo_coarse_tune,
            fine_tune: dcxo_fine_tune,
        } = config.external_clock_config
        {
            self.set_dcxo_tune(dcxo_coarse_tune, dcxo_fine_tune).await?;
        }

        self.0
            .modify_register(regs::Register::RefDivideConfig1, |val| {
                let mut reg = regs::ref_divide_config_1::Register::new_with_raw_value(val);
                reg.set_should_be_ones(u2::new(0b11));
                reg.raw_value()
            })
            .await?;
        self.0
            .modify_register(regs::Register::RefDivideConfig2, |val| {
                let mut reg = regs::ref_divide_config_2::Register::new_with_raw_value(val);
                reg.set_should_be_ones_1(u3::new(0b111));
                reg.set_should_be_ones_0(u2::new(0b11));
                reg.raw_value()
            })
            .await?;

        self.0
            .write_register(
                regs::Register::ClockEnable,
                regs::clock_enable::Register::builder()
                    .with_xo_bypass(config.external_clock_config.into())
                    .with_clock_enable_dflt(true)
                    .with_digital_power_up(true)
                    .with_bbpll_enable(true)
                    .build()
                    .raw_value(),
            )
            .await?;

        // Configure BB reference clock first.
        self.0
            .modify_register(regs::Register::ClockCtrl, |val| {
                let mut reg = regs::clock_ctrl::Register::new_with_raw_value(val);
                reg.set_scaler(config.clock.ref_clk_scalers.bb_refclk);
                reg.raw_value()
            })
            .await?;

        self.0
            .write_register(regs::Register::FractBbFreqWord2, 0x12)
            .await?;
        self.0
            .write_register(regs::Register::FractBbFreqWord3, 0x34)
            .await?;

        let clocks = self
            .0
            .setup_trx_clock_chain(
                config.reference_clk_rate,
                &config.clock,
                config.elna.as_ref(),
                &config.gain_control,
                &config.aux_adc,
            )
            .await?;

        match &config.transceiver_channel_mode {
            config::TransceiverChannelMode::Dual => {
                self.enable_rx_path(true, true).await?;
                self.enable_tx_path(true, true).await?;
            }
            config::TransceiverChannelMode::Single { rx, tx } => {
                let (rx1en, rx2en, tx1en, tx2en) = match (rx, tx) {
                    (ReceiverId::Rx1, TransmitterId::Tx1) => (true, false, true, false),
                    (ReceiverId::Rx1, TransmitterId::Tx2) => (true, false, false, true),
                    (ReceiverId::Rx2, TransmitterId::Tx1) => (false, true, true, false),
                    (ReceiverId::Rx2, TransmitterId::Tx2) => (false, true, false, true),
                };
                self.enable_rx_path(rx1en, rx2en).await?;
                self.enable_tx_path(tx1en, tx2en).await?;
            }
        }
        self.0
            .modify_register(regs::Register::InputSelect, |val| {
                let mut reg = regs::input_select::Register::new_with_raw_value(val);
                reg.set_tx_config(config.tx_path);
                reg.set_rx_config(config.rx_path);
                reg.raw_value()
            })
            .await?;

        self.setup_digital_ports(&config.digital_interface).await?;
        self.0.setup_auxadc(&clocks, &config.aux_adc).await?;

        let ref_clock_cycles_per_us = (config.reference_clk_rate / 1_000_000) - 1;
        self.0
            .write_register(
                regs::Register::ReferenceClockCycles,
                u7::new(
                    ref_clock_cycles_per_us
                        .clamp(0, u7::MAX.value() as u32)
                        .as_u8(),
                )
                .value(),
            )
            .await?;

        if let Some(elna) = &config.elna {
            self.setup_external_lna(elna).await?;
        }

        self.configure_rx_synth_clock_divider(config.clock.ref_clk_scalers.rx_synth)
            .await?;
        self.configure_tx_synth_clock_divider(config.clock.ref_clk_scalers.tx_synth)
            .await?;

        let fdd = DivisionDuplexMode::Fdd == config.division_duplex_config.mode();

        // Enable FDD mode during calibrations just like in the C driver.
        if !fdd {
            self.0
                .modify_register(regs::Register::ParallelPortConf3, |val| {
                    let mut reg = regs::parallel_port_conf_3::Register::new_with_raw_value(val);
                    reg.set_duplex_mode(regs::parallel_port_conf_3::Duplex::Full);
                    reg.raw_value()
                })
                .await?;
        }

        self.generic_tx_rx_synth_cp_calibration(config.reference_clk_rate, fdd, false)
            .await?;
        self.generic_tx_rx_synth_cp_calibration(config.reference_clk_rate, fdd, true)
            .await?;

        // Mirrors `ad9361_rfpll_vco_init` in the C driver: the FDD synth LUT is only used when
        // in FDD mode *and* not independent-mode *and* the RX/TX LOs actually differ. If the
        // two LOs are equal (as in this driver's typical single-LO setups), the TDD LUT is
        // used even in FDD mode -- using the RX/TX duplex mode alone is not enough.
        let rf_lo_differs = config.clock.rx.lo != config.clock.tx.lo;
        let lut_type = match &config.division_duplex_config {
            DivisionDuplexConfig::Fdd {
                independent_mode: false,
            } if rf_lo_differs => SynthLutType::Fdd,
            _ => SynthLutType::Tdd,
        };

        if let clocks::LocalOscClockConfig::Internal(rf_pll_config) = &config.clock.rx.lo {
            self.setup_rx_pll(lut_type, rf_pll_config, &clocks).await?;
        }

        // Always load the gain table for the configured frequency so we also have a value for the TX
        // Quad calibration.
        let gain_table_for_tx_quad = self.load_initial_rx_gain_table(config, &clocks).await?;

        if let clocks::LocalOscClockConfig::Internal(rf_pll_config) = &config.clock.tx.lo {
            self.setup_tx_pll(lut_type, rf_pll_config, &clocks).await?;
        }

        self.load_mixer_subtable().await?;

        self.setup_gain_control(config.gain_table_type, &config.gain_control)
            .await?;

        self.0
            .update_rf_bandwidth(
                &clocks,
                config.rf_rx_bandwidth_hz,
                config.rf_tx_bandwidth_hz,
            )
            .await?;

        self.calibrate_bb_dc_offset().await?;

        self.calibrate_rf_dc_offset(clocks.rx().lo(), config)
            .await?;

        let tx_quad = match config.tx_quad_calib {
            Some(phase_config) => {
                if gain_table_for_tx_quad.0.is_none() {
                    return Err(InitError::NoGainTableIndexForTxQuadCalibrationFound);
                }
                Some(TxQuadCalibParams {
                    phase_config,
                    gain_table_index: gain_table_for_tx_quad,
                    phase_inversion_en: config.digital_interface.rx1rx2_phase_inversion
                        || config.digital_interface.pp2_config().invert_rx2(),
                })
            }
            None => None,
        };
        if let Some(quad) = &tx_quad {
            self.0
                .perform_tx_quad_calibration(
                    config.transceiver_channel_mode,
                    &clocks,
                    config.rf_tx_bandwidth_hz,
                    config.rf_rx_bandwidth_hz,
                    quad.phase_config,
                    quad.gain_table_index,
                    quad.phase_inversion_en,
                )
                .await?;
        }

        self.setup_tracking_control(config).await?;

        // Restore user specified duplex configuration after all calibration steps.
        if !fdd {
            self.0
                .write_register(
                    regs::Register::ParallelPortConf3,
                    config.digital_interface.pp3_config().raw_value(),
                )
                .await?;
        }

        self.setup_ensm_mode(&config.division_duplex_config, config)
            .await?;

        self.0
            .modify_register(regs::Register::TxAttenOffset, |val| {
                let mut reg = regs::tx_atten_offset::Register::new_with_raw_value(val);
                reg.set_mask_clr_atten_update(false);
                reg.raw_value()
            })
            .await?;

        self.set_tx_attenuation(config).await?;

        self.0.setup_rssi(&clocks, &config.rssi, false).await?;

        self.0
            .modify_register(regs::Register::BbPll, |val| {
                let mut reg = regs::clk_bb_pll::Register::new_with_raw_value(val);
                match config.clkout_mode {
                    Some(mode) => {
                        reg.set_clkout_enable(true);
                        reg.set_clk_out_select(mode);
                    }
                    None => {
                        reg.set_clkout_enable(false);
                    }
                }
                reg.raw_value()
            })
            .await?;

        if let config::TxMonitorConfig::Enabled(txmon_config) = &config.tx_monitoring {
            self.setup_txmon(txmon_config).await?;
        }

        let ensm_state_reg = regs::state::Register::new_with_raw_value(
            self.0.read_register(regs::Register::State).await?,
        );

        let mut state = Ad9361State {
            current_ensm_state: if let Ok(ensm_state) = ensm_state_reg.ensm_state() {
                ensm_state.into()
            } else {
                return Err(InitError::InvalidEnsmState);
            },
            ensm_pin_control: config.ensm_enable_txnrx_control,
        };
        state.current_ensm_state = self.setup_ensm_state(config).await?;

        // Calibration is done now, so it's safe to enable the RX/TX FIR at the configured target
        // ratio. See configure_rx_hb_clock_chain/configure_tx_clock_chain for why it was
        // bypassed until this point.
        self.0.enable_configured_fir(&config.clock).await?;

        Ok((
            Ad9361 {
                inner: self.0,
                config: Ad9361Config {
                    external_clock_config: config.external_clock_config,
                    #[cfg(feature = "axi-tune")]
                    num_axi_channels: match config.transceiver_channel_mode {
                        TransceiverChannelMode::Dual => u4::new(4),
                        TransceiverChannelMode::Single { .. } => u4::new(2),
                    },
                    reference_clk_rate: config.reference_clk_rate,
                    clock: config.clock.clone(),
                    rf_rx_bandwidth_hz: config.rf_rx_bandwidth_hz,
                    rf_tx_bandwidth_hz: config.rf_tx_bandwidth_hz,
                    gain_control: config.gain_control.clone(),
                    elna: config.elna.clone(),
                    aux_adc: config.aux_adc.clone(),
                    rssi: config.rssi.clone(),
                    transceiver_channel_mode: config.transceiver_channel_mode,
                    tx_quad,
                    tx_fir_taps: None,
                    rx_fir_taps: None,
                },
                state,
            },
            clocks,
        ))
    }

    async fn setup_ensm_state(
        &mut self,
        config: &config::ConfigRaw,
    ) -> Result<EnsmState, Spi::Error> {
        let mut reg = regs::ensm_config_1::Register::ZERO
            .with_level_mode(!config.ensm_enable_pin_pulse_mode)
            .with_enable_ensm_pin_ctrl(config.ensm_enable_txnrx_control)
            .with_to_alert(true);
        if let config::DivisionDuplexConfig::Tdd {
            enable_txmon: txmon,
            use_dual_synth: _,
        } = config.division_duplex_config
        {
            reg.set_enable_rx_data_port_for_cal(txmon)
        }
        let target_state = match config.division_duplex_config.mode() {
            DivisionDuplexMode::Fdd => regs::EnsmState::Fdd,
            DivisionDuplexMode::Tdd => regs::EnsmState::Rx,
        };
        match target_state {
            regs::EnsmState::Fdd => {
                reg.set_force_tx_on(true);
            }
            regs::EnsmState::Rx => {
                reg.set_force_rx_on(true);
            }
            _ => unreachable!(),
        }
        if let config::DivisionDuplexConfig::Tdd {
            enable_txmon: _,
            use_dual_synth,
        } = config.division_duplex_config
            && !config.ensm_enable_txnrx_control
            && !use_dual_synth
            && target_state == regs::EnsmState::Rx
        {
            self.0
                .modify_register(regs::Register::EnsmConfig2, |val| {
                    let mut reg = regs::ensm_config_2::Register::new_with_raw_value(val);
                    reg.set_txnrx_spi_ctrl(regs::ensm_config_2::TxNRxSpiCtrl::Rx);
                    reg.raw_value()
                })
                .await?;
        }
        self.0
            .write_register(regs::Register::EnsmConfig1, reg.raw_value())
            .await?;
        // TODO: The C driver does some manual gain control specific stuff. not implemented yet.
        Ok(target_state.into())
    }

    async fn setup_digital_ports(
        &mut self,
        config: &config::DigitalInterfaceConfig,
    ) -> Result<(), Spi::Error> {
        self.setup_digital_ports_impl(config).await
    }

    async fn enable_rx_path(
        &mut self,
        enable_rx_1: bool,
        enable_rx_2: bool,
    ) -> Result<(), Spi::Error> {
        self.enable_rx_path_impl(enable_rx_1, enable_rx_2).await
    }

    async fn enable_tx_path(
        &mut self,
        enable_tx_1: bool,
        enable_tx_2: bool,
    ) -> Result<(), Spi::Error> {
        self.enable_tx_path_impl(enable_tx_1, enable_tx_2).await
    }

    async fn set_dcxo_tune(&mut self, dcxo_coarse: u6, dcxo_fine: u13) -> Result<(), Spi::Error> {
        self.set_dcxo_tune_impl(dcxo_coarse, dcxo_fine).await
    }

    async fn setup_external_lna(
        &mut self,
        config: &config::ExternalLnaConfig,
    ) -> Result<(), Spi::Error> {
        self.setup_external_lna_impl(config).await
    }

    async fn setup_auxdac(
        &mut self,
        aux_dac_config: &config::AuxDacConfig,
    ) -> Result<AuxDacValuesMv, Spi::Error> {
        self.setup_auxdac_impl(aux_dac_config).await
    }

    /// Set up general purpose output (GPO) pins according to the provided configuration.
    async fn setup_gpo(&mut self, gpo_config: &config::GpoConfig) -> Result<(), Spi::Error> {
        self.setup_gpo_impl(gpo_config).await
    }

    async fn setup_txmon(&mut self, config: &config::TxMonConfigParams) -> Result<(), Spi::Error> {
        self.0
            .write_register(
                regs::Register::TpmModeEnable,
                regs::tpm_mode_enable::Register::ZERO
                    .with_one_shot_mode(config.one_shot_mode)
                    .with_tx_mon_duration(u4::new(
                        (config.tx_mon_duration / 16).ilog(2).min(u4::MAX.as_u32()) as u8,
                    ))
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::TxMonDelay,
                (config.tx_mon_delay.value() & 0xff) as u8,
            )
            .await?;
        self.0
            .modify_register(regs::Register::TxLevelThresh, |val| {
                let mut reg = regs::tx_level_thresh::Register::new_with_raw_value(val);
                reg.set_tx_mon_delay_counter(u2::new(
                    (config.tx_mon_delay.value() >> 8 & 0b11) as u8,
                ));
                reg.raw_value()
            })
            .await?;
        self.0
            .write_register(
                regs::Register::TxMon1Config,
                regs::tx_mon_config::Register::builder()
                    .with_tx_mon_lo_cm(config.tx1_mon_lo_cm)
                    .with_tx_mon_gain(config.tx1_mon_front_end_gain)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::TxMon2Config,
                regs::tx_mon_config::Register::builder()
                    .with_tx_mon_lo_cm(config.tx2_mon_lo_cm)
                    .with_tx_mon_gain(config.tx2_mon_front_end_gain)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::TxAttenThresh,
                (config.low_high_gain_threshold_md_b / 250).max(u8::MAX.as_u32()) as u8,
            )
            .await?;
        self.0
            .write_register(
                regs::Register::TxMonHighGain,
                regs::tx_mon_high_gain::Register::builder()
                    .with_tx_mon_high_gain(config.high_gain_d_b)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::TxMonLowGain,
                regs::tx_mon_low_gain::Register::builder()
                    .with_tx_mon_track(config.tx_mon_track)
                    .with_tx_mon_low_gain(config.low_gain_d_b)
                    .build()
                    .raw_value(),
            )
            .await?;
        Ok(())
    }

    async fn set_tx_attenuation(&mut self, config: &config::ConfigRaw) -> Result<(), Spi::Error> {
        // Assume validated config.

        // Scale to 0.25 dB / LSB.
        let atten_reg_val = config.tx_attenuation_md_b / 250;
        self.0
            .modify_register(regs::Register::Tx2DigAtten, |val| {
                regs::tx2_dig_atten::Register::new_with_raw_value(val)
                    .with_immediately_update_tpc_atten(false)
                    .raw_value()
            })
            .await?;
        let mut buf: [u8; 4] = [0; 4];
        let (update_tx1, update_tx2, attenuate_inactive) = match config.transceiver_channel_mode {
            TransceiverChannelMode::Dual => (true, true, false),
            TransceiverChannelMode::Single { rx: _, tx } => {
                (tx == TransmitterId::Tx1, tx == TransmitterId::Tx2, true)
            }
        };
        if update_tx1 {
            self.0
                .write_multiple_bytes_decrementing(&mut buf, regs::Register::Tx1Atten1, |payload| {
                    payload[0] = ((atten_reg_val >> 8) & 0xff) as u8;
                    payload[1] = (atten_reg_val & 0xff) as u8;
                })
                .await?;
        }
        if update_tx2 {
            self.0
                .write_multiple_bytes_decrementing(&mut buf, regs::Register::Tx2Atten1, |payload| {
                    payload[0] = ((atten_reg_val >> 8) & 0xff) as u8;
                    payload[1] = (atten_reg_val & 0xff) as u8;
                })
                .await?;
        }
        if attenuate_inactive {
            let inactive_atten_reg_val = 89750 / 250;
            self.0
                .write_multiple_bytes_decrementing(
                    &mut buf,
                    if update_tx1 {
                        regs::Register::Tx2Atten1
                    } else {
                        regs::Register::Tx1Atten1
                    },
                    |payload| {
                        payload[0] = ((inactive_atten_reg_val >> 8) & 0xff) as u8;
                        payload[1] = (inactive_atten_reg_val & 0xff) as u8;
                    },
                )
                .await?;
        }
        self.0
            .modify_register(regs::Register::Tx2DigAtten, |val| {
                regs::tx2_dig_atten::Register::new_with_raw_value(val)
                    .with_immediately_update_tpc_atten(true)
                    .raw_value()
            })
            .await?;
        Ok(())
    }

    async fn setup_ensm_mode(
        &mut self,
        division_duplex_config: &config::DivisionDuplexConfig,
        config: &config::ConfigRaw,
    ) -> Result<(), Spi::Error> {
        self.0
            .write_register(
                regs::Register::EnsmMode,
                regs::ensm_mode::Register::builder()
                    .with_fdd_mode(division_duplex_config.mode() == DivisionDuplexMode::Fdd)
                    .build()
                    .raw_value(),
            )
            .await?;

        self.0
            .modify_register(regs::Register::EnsmConfig2, |val| {
                let mut cfg2 = regs::ensm_config_2::Register::new_with_raw_value(val);
                match division_duplex_config {
                    config::DivisionDuplexConfig::Fdd {
                        independent_mode: fdd_independent_mode,
                    } => {
                        cfg2.set_dual_synth_mode(true);
                        if *fdd_independent_mode {
                            cfg2.set_fdd_external_ctrl_enable(true);
                        }
                    }
                    config::DivisionDuplexConfig::Tdd {
                        enable_txmon: _,
                        use_dual_synth: tdd_use_dual_synth,
                    } => {
                        if *tdd_use_dual_synth {
                            cfg2.set_synth_enable_pin_ctrl_mode(false);
                            cfg2.set_dual_synth_mode(true);
                        } else {
                            cfg2.set_synth_enable_pin_ctrl_mode(config.ensm_enable_txnrx_control);
                        }
                    }
                }
                cfg2.raw_value()
            })
            .await?;

        Ok(())
    }

    async fn setup_tracking_control(
        &mut self,
        config: &config::ConfigRaw,
    ) -> Result<(), Spi::Error> {
        self.0
            .write_register(
                regs::Register::CalibrationConfig2,
                regs::calibration_config_2::Register::builder()
                    .with_soft_reset(false)
                    .with_should_be_ones(u2::new(0b11))
                    .with_k_exp_phase(u5::new(0x15))
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::CalibrationConfig3,
                regs::calibration_config_3::Register::builder()
                    .with_prevent_pos_loop_gain(true)
                    .with_k_exp_amplitude(u5::new(0x15))
                    .build()
                    .raw_value(),
            )
            .await?;

        self.0
            .write_register(
                regs::Register::DcOffsetConfig2,
                regs::dc_offset_config2::Register::builder()
                    .with_use_wait_counter_for_rf_dc_init_cal(true)
                    .with_dc_offset_update(config.dc_offset_tracking_update_event)
                    .with_enable_bb_dc_offset_tracking(true)
                    .with_enable_rf_offset_tracking(true)
                    .with_enable_fast_settle_mode(false)
                    .with_reset_acc_on_gain_change(false)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .modify_register(regs::Register::RxQuadGain2, |val| {
                let mut reg = regs::rx_quad_gain2::Register::new_with_raw_value(val);
                if config.qec_tracking_slow_mode {
                    reg.set_correction_word_decimation_m(u3::new(4))
                } else {
                    reg.set_correction_word_decimation_m(u3::new(0))
                }
                reg.raw_value()
            })
            .await?;

        let (ch1_enable, ch2_enable) = match &config.transceiver_channel_mode {
            TransceiverChannelMode::Dual => (true, true),
            TransceiverChannelMode::Single { rx, tx: _ } => {
                (*rx == ReceiverId::Rx1, *rx == ReceiverId::Rx2)
            }
        };
        self.0
            .write_register(
                regs::Register::CalibrationConfig1,
                regs::calibration_config_1::Register::builder()
                    .with_enable_phase_corr(true)
                    .with_enable_gain_corr(true)
                    .with_free_run_mode(true)
                    .with_use_settle_count_for_dc_cal_wait(false)
                    .with_fixed_dc_cal_wait_time(false)
                    .with_enable_corr_word_decimation(true)
                    .with_enable_tracking_mode_ch1(ch1_enable)
                    .with_enable_tracking_mode_ch2(ch2_enable)
                    .build()
                    .raw_value(),
            )
            .await?;
        Ok(())
    }

    async fn setup_gain_control(
        &mut self,
        gain_table_type: RxGainTableType,
        config: &config::GainControl,
    ) -> Result<(), Spi::Error> {
        self.setup_gain_control_generic(gain_table_type, config)
            .await?;

        if let Some(auto_config) = config.auto() {
            let decrement_step_size_small_lpf_gain_full_table_case_2 =
                config.fast().map_or(u3::ZERO, |v| {
                    v.decrement_step_small_lpf_gain_full_table_case_2
                });
            self.setup_auto_gain_control_generic(
                auto_config,
                decrement_step_size_small_lpf_gain_full_table_case_2,
            )
            .await?;
        }

        if let Some(fast_config) = config.fast() {
            self.setup_fast_agc(fast_config).await?;
        }

        Ok(())
    }

    async fn setup_auto_gain_control_generic(
        &mut self,
        config: &config::GainControlAutoConfig,
        decrement_step_size_small_lpf_gain_full_table_case_2: u3,
    ) -> Result<(), Spi::Error> {
        self.0
            .modify_register(regs::Register::AgcLockLevel, |val| {
                let mut reg = regs::agc_lock_level::Register::new_with_raw_value(val);
                reg.set_agc_lock_level_fast_agc_inner_high_thresh_slow(
                    config.inner_thresh_high_negative_dbfs,
                );
                reg.raw_value()
            })
            .await?;

        self.0
            .write_register(
                regs::Register::AgcInnerLowThresh,
                regs::agc_inner_low_thresh::Register::builder()
                    .with_agc_inner_low_thresh(config.inner_thresh_low_negative_dbfs)
                    .with_prevent_gain_inc(config.prevent_gain_inc_lmt_adc_small_overload)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::OuterPowerThreshs,
                regs::outer_power_threshs::Register::builder()
                    .with_agc_offset_of_outer_high_to_inner_high(
                        config.outer_thresh_offset_to_inner_high,
                    )
                    .with_agc_offset_to_outer_low_to_inner_low(
                        config.outer_thresh_offset_to_inner_low,
                    )
                    .build()
                    .raw_value(),
            )
            .await?;

        self.0
            .write_register(
                regs::Register::GainStep2,
                regs::gain_step2::Register::builder()
                    .with_agc_outer_high_thresh_exed_stp_size(config.outer_thresh_high_dec_steps)
                    .with_agc_outer_low_thresh_exed_stp_size(config.outer_thresh_low_inc_steps)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::GainStep1,
                regs::gain_step1::Register::builder()
                    .with_immed_gain_change_if_lg_lmt_overload(
                        config.immed_gain_change_if_large_lmt_overload,
                    )
                    .with_immed_gain_change_if_lg_adc_overload(
                        config.immed_gain_change_if_large_adc_overload,
                    )
                    .with_agc_inner_high_thresh_exed_stp_size(config.inner_thresh_high_dec_steps)
                    .with_agc_inner_low_thresh_exed_stp_size(config.inner_thresh_low_inc_steps)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::AdcOverloadCounters,
                regs::adc_overload_counters::Register::builder()
                    .with_large_adc_overload_exed_counter(config.adc_large_overload_exceed_counter)
                    .with_small_adc_overload_exed_counter(config.adc_small_overload_exceed_counter)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::GainStepConfig2,
                regs::gain_step_config2::Register::builder()
                    .with_decrement_stp_size_for_small_lpf_gain_change(
                        decrement_step_size_small_lpf_gain_full_table_case_2,
                    )
                    .with_large_lpf_gain_step(
                        config.decrement_step_size_large_lpf_or_full_table_case_1,
                    )
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::LmtOverloadCounters,
                regs::lmt_overload_counters::Register::builder()
                    .with_large_lmt_overload_exed_counter(config.lmt_overload_large_exceed_counter)
                    .with_small_lmt_overload_exed_counter(config.lmt_overload_small_exceed_counter)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .modify_register(regs::Register::GainStepConfig1, |val| {
                let mut reg = regs::gain_step_config1::Register::new_with_raw_value(val);
                reg.set_dec_stp_size_for_large_lmt_overload(
                    config.decrement_step_size_large_lmt_overload_full_table_case_3,
                );
                reg.raw_value()
            })
            .await?;
        self.0
            .write_register(
                regs::Register::DigitalSaturationCounter,
                regs::digital_sat_counter::Register::builder()
                    .with_double_gain_counter(false)
                    .with_enable_sync_for_gain_counter(config.enable_sync_for_gain_counter)
                    .with_dig_saturation_exed_counter(config.dig_saturation_exceed_counter)
                    .build()
                    .raw_value(),
            )
            .await?;
        Ok(())
    }

    async fn setup_gain_control_generic(
        &mut self,
        gain_table_type: RxGainTableType,
        config: &config::GainControl,
    ) -> Result<(), Spi::Error> {
        self.0
            .write_register(
                regs::Register::AgcConfig1,
                regs::agc_config_1::Register::builder()
                    .with_dec_pwr_for_low_pwr(true)
                    .with_dec_pwr_for_lock_level(true)
                    .with_dec_pwr_for_gain_lock_exit(true)
                    .with_slow_attack_hybrid_mode(
                        config.rx1_mode() == GainControlMode::AutoHybrid
                            || config.rx2_mode() == GainControlMode::AutoHybrid,
                    )
                    .with_rx2_gain_ctrl_setup(config.rx2_mode().into())
                    .with_rx1_gain_ctrl_setup(config.rx1_mode().into())
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .modify_register(regs::Register::AgcConfig2, |val| {
                let mut reg = regs::agc_config_2::Register::new_with_raw_value(val);
                if config.common.digital_config.is_some() {
                    reg.set_dig_gain_en(true);
                }
                if let Some(manual_config) = config.manual() {
                    reg.set_man_gain_ctrl_rx2(manual_config.rx2_ctrl_input);
                    reg.set_man_gain_ctrl_rx1(manual_config.rx1_ctrl_input);
                }
                reg.raw_value()
            })
            .await?;

        let clamped_adc_ovr_sample_size = config.common.adc_ovr_sample_size.clamp(1, 8);
        let mut agc_config_3_val = regs::agc_config_3::Register::ZERO
            .with_adc_overrange_sample_size(u3::new(clamped_adc_ovr_sample_size - 1));
        if let Some(manual_config) = config.manual() {
            agc_config_3_val
                .set_manual_incr_step_size(u3::new(manual_config.inc_gain_step.clamp(1, 8) - 1));
            if let Some(split_table_mode) = manual_config.split_table_mode {
                match split_table_mode {
                    config::SplitTableManualMode::Agc => {
                        agc_config_3_val.set_use_agc_for_lmtlpf_gain(true)
                    }
                    config::SplitTableManualMode::OnlyInLpf => {
                        agc_config_3_val.set_use_agc_for_lmtlpf_gain(false);
                        agc_config_3_val.set_incdec_lmt_gain(false);
                    }
                    config::SplitTableManualMode::OnlyInLmt => {
                        agc_config_3_val.set_use_agc_for_lmtlpf_gain(false);
                        agc_config_3_val.set_incdec_lmt_gain(true);
                    }
                }
            }
        }
        self.0
            .write_register(regs::Register::AgcConfig3, agc_config_3_val.raw_value())
            .await?;

        if let Some(manual_config) = config.manual() {
            self.0
                .modify_register(regs::Register::PeakWaitTime, |val| {
                    let mut reg = regs::peak_wait_time::Register::new_with_raw_value(val);
                    reg.set_manual_decr_gain_stp_size(u3::new(
                        manual_config.dec_gain_step.clamp(1, 8) - 1,
                    ));
                    reg.raw_value()
                })
                .await?;
        }

        if let Some(digital_config) = &config.common.digital_config {
            self.0
                .write_register(
                    regs::Register::DigitalGain,
                    regs::digital_gain::Register::builder()
                        .with_dig_gain_step_size(u3::new(
                            digital_config.dig_gain_step_size.clamp(1, 8) - 1,
                        ))
                        .with_maximum_digital_gain(digital_config.max_dig_gain)
                        .build()
                        .raw_value(),
                )
                .await?;
        }
        // Sanitization of config takes care of ensuring valid values.
        self.0
            .write_register(
                regs::Register::AdcSmallOverloadThresh,
                config.common.adc_small_overload_thresh,
            )
            .await?;
        self.0
            .write_register(
                regs::Register::AdcLargeOverloadThresh,
                config.common.adc_large_overload_thresh,
            )
            .await?;

        self.0
            .modify_register(regs::Register::SmallLmtOverloadThresh, |val| {
                let mut reg = regs::small_lmt_overload_thresh::Register::new_with_raw_value(val);
                reg.set_small_lmt_overload_thresh(u6::new(
                    ((config.common.lmt_overload_low_thresh_mv_peak / 16).saturating_sub(1))
                        .clamp(0, 63) as u8,
                ));
                reg.raw_value()
            })
            .await?;
        self.0
            .write_register(
                regs::Register::LargeLmtOverloadThresh,
                ((config.common.lmt_overload_high_thresh_mv_peak / 16).saturating_sub(1))
                    .clamp(0, 63) as u8,
            )
            .await?;

        // This is taken from the C library. I do not know why the values are like that.
        if gain_table_type == RxGainTableType::Split {
            self.0
                .write_register(regs::Register::Rx1ManualLpfGainOrAgcCurrentLpf, 0x58)
                .await?;
            self.0
                .write_register(regs::Register::Rx2ManualLpfGainOrAgcCurentLpf, 0x18)
                .await?;
            self.0
                .write_register(regs::Register::FastInitialLmtGainLimit, 0x27)
                .await?;
        }

        self.0
            .write_register(regs::Register::Rx1ManualDigitalOrForcedGain, 0x00)
            .await?;
        self.0
            .write_register(regs::Register::Rx2ManualDigitalOrForcedGain, 0x00)
            .await?;

        self.0
            .write_register(
                regs::Register::FastLowPowerThresh,
                config.common.low_power_thresh.clamp(0, 64) * 2,
            )
            .await?;
        self.0
            .write_register(regs::Register::TxSymbolAttenConfig, 0x0)
            .await?;

        self.0
            .modify_register(regs::Register::DecPowerMeasureDuration, |val| {
                regs::dec_power_measure_duration::Register::new_with_raw_value(val)
                    .with_dec_power_measurement_source(config.common.dec_pwr_meas_source)
                    .with_enable_dec_pwr_meas(true)
                    .with_dec_power_measurement_duration(
                        config.common.dec_pow_measurement_duration.0,
                    )
                    .raw_value()
            })
            .await?;
        Ok(())
    }

    async fn setup_fast_agc(
        &mut self,
        fast_config: &config::GainControlFastAutoConfig,
    ) -> Result<(), Spi::Error> {
        self.0
            .modify_register(regs::Register::FastConfig1, |val| {
                let mut reg = regs::fast_config_1::Register::new_with_raw_value(val);
                reg.set_enable_incr_gain(fast_config.allow_agc_gain_increase);
                reg.raw_value()
            })
            .await?;

        self.0
            .write_register(
                regs::Register::FastIncrementTime,
                fast_config.lp_thresh_increment_time,
            )
            .await?;

        let lp_thresh_inc_steps_reg = fast_config.lp_thresh_increment_steps.clamp(1, 8) - 1;
        self.0
            .modify_register(regs::Register::FastEnergyDetectCount, |val| {
                let mut reg = regs::fast_energy_detect_count::Register::new_with_raw_value(val);
                reg.set_increment_gain_stp_lpflmt(u3::new(lp_thresh_inc_steps_reg));
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastConfig2SettlingDelay, |val| {
                let mut reg = regs::fast_config_2_settling_delay::Register::new_with_raw_value(val);
                reg.set_enable_lmt_gain_inc_for_lock_level(
                    fast_config.lock_level_lmt_gain_increase,
                );
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastAgcllUpperLimit, |val| {
                let mut reg = regs::fast_agcll_upper_limit::Register::new_with_raw_value(val);
                reg.set_agcll_max_increase(fast_config.lock_level_gain_increase_upper_limit);
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastEnergyLostThresh, |val| {
                let mut reg = regs::fast_energy_lost_thresh::Register::new_with_raw_value(val);
                reg.set_post_lock_level_stp_size_for_lpf_table_full_table(
                    fast_config.lpf_final_settling_steps,
                );
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastStrongerSignalThresh, |val| {
                let mut reg = regs::fast_stronger_signal_thresh::Register::new_with_raw_value(val);
                reg.set_post_lock_level_stp_for_lmt_table(fast_config.lmt_final_settling_steps);
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastFinalOverRangeAndOptGain, |val| {
                let mut reg =
                    regs::fast_final_over_range_and_opt_gain::Register::new_with_raw_value(val);
                reg.set_final_over_range_count(fast_config.final_overrange_count);
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastConfig1, |val| {
                let mut reg = regs::fast_config_1::Register::new_with_raw_value(val);
                reg.set_enable_gain_inc_after_gain_lock(fast_config.gain_increase_after_gain_lock);
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastConfig2SettlingDelay, |val| {
                let mut reg = regs::fast_config_2_settling_delay::Register::new_with_raw_value(val);
                reg.set_use_last_lock_level_for_set_gain(
                    fast_config.use_last_lock_level_for_set_gain,
                );
                reg.raw_value()
            })
            .await?;
        self.0
            .modify_register(regs::Register::FastFinalOverRangeAndOptGain, |val| {
                let mut reg =
                    regs::fast_final_over_range_and_opt_gain::Register::new_with_raw_value(val);
                reg.set_optimize_gain_offset(fast_config.optimized_gain_offset);
                reg.raw_value()
            })
            .await?;

        // Fast config 1 configuration.
        self.0
            .modify_register(regs::Register::FastConfig1, |val| {
                regs::fast_config_1::Register::new_with_raw_value(val)
                    .with_goto_set_gain_if_exit_rx_state(
                        fast_config.gain_index_type_after_exit_rx_mode
                            == FastAgcTargetGainIndexType::SetGain,
                    )
                    .with_goto_optimized_gain_if_exit_rx_state(
                        fast_config.gain_index_type_after_exit_rx_mode
                            == FastAgcTargetGainIndexType::OptimizedGain,
                    )
                    .with_dont_unlock_gain_if_lg_adc_or_lmt_ovrg(
                        fast_config.suppress_gain_unlock_on_large_adc_large_lmt_overload,
                    )
                    .with_dont_unlock_gain_if_energy_lost(
                        fast_config.suppress_gain_unlock_on_energy_lost_threshold_exceeded,
                    )
                    .raw_value()
            })
            .await?;

        let en_agc_pin_used = fast_config.gain_index_type_on_en_agc_high.is_some();

        let any_gain_unlock_suppressed = fast_config
            .suppress_gain_unlock_on_energy_lost_threshold_exceeded
            || fast_config.suppress_gain_unlock_on_large_adc_large_lmt_overload
            || fast_config.suppress_gain_unlock_on_stronger_signal_threshold_exceeded;
        // Setting 0x0FB[D6] bit.
        self.0
            .modify_register(regs::Register::AgcConfig2, |val| {
                regs::agc_config_2::Register::new_with_raw_value(val)
                    .with_agc_gain_unlock_ctrl(en_agc_pin_used || any_gain_unlock_suppressed)
                    .raw_value()
            })
            .await?;

        // Gain unlock control: EN_AGC pin control
        if let Some(gain_index_type) = fast_config.gain_index_type_on_en_agc_high {
            match gain_index_type {
                FastAgcTargetGainIndexType::MaxGain => {
                    self.0
                        .modify_register(regs::Register::FastConfig2SettlingDelay, |val| {
                            let mut reg =
                                regs::fast_config_2_settling_delay::Register::new_with_raw_value(
                                    val,
                                );
                            reg.set_goto_max_gain_or_opt_gain_if_en_agc_high(true);
                            reg.raw_value()
                        })
                        .await?;
                    self.0
                        .modify_register(regs::Register::FastConfig1, |val| {
                            let mut reg = regs::fast_config_1::Register::new_with_raw_value(val);
                            reg.set_goto_set_gain_if_exit_rx_state(false);
                            reg.set_goto_opt_gain_if_energy_lost_or_en_agc_high(false);
                            reg.raw_value()
                        })
                        .await?;
                }
                FastAgcTargetGainIndexType::SetGain => {
                    self.0
                        .modify_register(regs::Register::FastConfig2SettlingDelay, |val| {
                            let mut reg =
                                regs::fast_config_2_settling_delay::Register::new_with_raw_value(
                                    val,
                                );
                            reg.set_goto_max_gain_or_opt_gain_if_en_agc_high(false);
                            reg.raw_value()
                        })
                        .await?;
                    self.0
                        .modify_register(regs::Register::FastConfig1, |val| {
                            let mut reg = regs::fast_config_1::Register::new_with_raw_value(val);
                            reg.set_goto_set_gain_if_en_agc_high(true);
                            reg.raw_value()
                        })
                        .await?;
                }
                FastAgcTargetGainIndexType::OptimizedGain => {
                    self.0
                        .modify_register(regs::Register::FastConfig2SettlingDelay, |val| {
                            let mut reg =
                                regs::fast_config_2_settling_delay::Register::new_with_raw_value(
                                    val,
                                );
                            reg.set_goto_max_gain_or_opt_gain_if_en_agc_high(true);
                            reg.raw_value()
                        })
                        .await?;
                    self.0
                        .modify_register(regs::Register::FastConfig1, |val| {
                            let mut reg = regs::fast_config_1::Register::new_with_raw_value(val);
                            reg.set_goto_set_gain_if_en_agc_high(false);
                            reg.set_goto_opt_gain_if_energy_lost_or_en_agc_high(true);
                            reg.raw_value()
                        })
                        .await?;
                }
                FastAgcTargetGainIndexType::NoGainChange => {
                    self.0
                        .modify_register(regs::Register::FastConfig2SettlingDelay, |val| {
                            let mut reg =
                                regs::fast_config_2_settling_delay::Register::new_with_raw_value(
                                    val,
                                );
                            reg.set_goto_max_gain_or_opt_gain_if_en_agc_high(false);
                            reg.raw_value()
                        })
                        .await?;
                    self.0
                        .modify_register(regs::Register::FastConfig1, |val| {
                            let mut reg = regs::fast_config_1::Register::new_with_raw_value(val);
                            reg.set_goto_opt_gain_if_energy_lost_or_en_agc_high(false);
                            reg.raw_value()
                        })
                        .await?;
                }
            }
        }

        // Set 0x014[D1] bit.
        if !en_agc_pin_used || any_gain_unlock_suppressed {
            self.0
                .modify_register(regs::Register::EnsmConfig1, |val| {
                    regs::ensm_config_1::Register::new_with_raw_value(val)
                        .with_auto_gain_lock(true)
                        .raw_value()
                })
                .await?;
        }

        // ADC overload gain unlock can be suppressed individually.
        self.0
            .modify_register(regs::Register::FastLowPowerThresh, |val| {
                let mut reg = regs::fast_low_power_thresh::Register::new_with_raw_value(val);
                reg.set_dont_unlock_gain_if_adc_ovrg(
                    fast_config.suppress_gain_unlock_on_adc_overload,
                );
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastStrongSignalFreeze, |val| {
                let mut reg = regs::fast_strong_signal_freeze::Register::new_with_raw_value(val);
                reg.set_dont_unlock_gain_if_stronger_signal(
                    fast_config.suppress_gain_unlock_on_stronger_signal_threshold_exceeded,
                );
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastStrongerSignalThresh, |val| {
                let mut reg = regs::fast_stronger_signal_thresh::Register::new_with_raw_value(val);
                reg.set_stronger_signal_thresh(fast_config.rst_gla_stronger_sig_thresh_above_ll);
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastEnergyLostThresh, |val| {
                let mut reg = regs::fast_energy_lost_thresh::Register::new_with_raw_value(val);
                reg.set_energy_lost_thresh(fast_config.rst_gla_energy_lost_sig_thresh_below_ll);
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::FastGainLockExitCount, |val| {
                let mut reg = regs::fast_gain_lock_exit_count::Register::new_with_raw_value(val);
                reg.set_gain_lock_exit_count(
                    fast_config.energy_lost_stronger_sig_gain_lock_exit_cnt,
                );
                reg.raw_value()
            })
            .await?;

        // Power measurement duration in state 5
        let power_meas_duration = (fast_config.power_measurement_duration_in_state5 / 16)
            .ilog2()
            .min(15) as u8;
        self.0
            .modify_register(regs::Register::Rx1ManualLpfGainOrAgcCurrentLpf, |val| {
                let mut reg = regs::rx1_manual_lpf_gain::Register::new_with_raw_value(val);
                reg.set_power_meas_in_state_5(u3::new(power_meas_duration & 0b111));
                reg.raw_value()
            })
            .await?;

        // Also write MSB to Rx1ManualLmtFullGain (register 0x109)
        self.0
            .modify_register(regs::Register::Rx1ManualLmtFullGain, |val| {
                let mut reg = regs::rx1_manual_lmt_full_gain::Register::new_with_raw_value(val);
                reg.set_power_meas_in_state_5_msb((power_meas_duration >> 3) == 1);
                reg.raw_value()
            })
            .await?;

        Ok(())
    }

    // Returns the gain table index for TX QUAD calibration, which generally is the highest index
    // with the TIA bit 1.
    async fn load_initial_rx_gain_table(
        &mut self,
        config: &config::ConfigRaw,
        clocks: &clocks::Clocks,
    ) -> Result<GainTableIndexForTxQuad, Spi::Error> {
        const TIA_BIT_MASK: u8 = 0x20;
        const LPF_BIT_MASK: u8 = 0x1F;
        let mut gain_config_for_tx_quad = GainTableIndexForTxQuad(None);
        let gain_table = RxGainTable::new_for_lo_frequency(clocks.rx().lo());
        let gain_lut = gain_table.gain_lut(config.gain_table_type);
        // For the full table case, TIA needs to be 1 and LPF needs to be 0. In the split table
        // case, we only care about TIA being 1.
        let lpf_tia_mask = match config.gain_table_type {
            RxGainTableType::Full => LPF_BIT_MASK | TIA_BIT_MASK,
            RxGainTableType::Split => TIA_BIT_MASK,
        };
        self.0
            .modify_register(regs::Register::AgcConfig2, |val| {
                let mut reg = regs::agc_config_2::Register::new_with_raw_value(val);
                reg.set_agc_use_full_gain_table(config.gain_table_type == RxGainTableType::Full);
                reg.raw_value()
            })
            .await?;
        self.0
            .write_register(
                regs::Register::MaxLmtFullGain,
                regs::max_lmt_full_gain::Register::builder()
                    .with_maximum_full_tablelmt_table_index(
                        // This should never panic, because there are only sizes 77 and 41.
                        u7::try_new((gain_lut.len() - 1) as u8)
                            .expect("Gain LUT size unexpectedly large"),
                    )
                    .build()
                    .raw_value(),
            )
            .await?;

        for (index, lut_entry) in gain_lut.iter().enumerate() {
            self.0
                .write_register(
                    regs::Register::GainTableAddress,
                    regs::gain_table_address::Register::builder()
                        .with_gain_table_address(u7::new(index as u8))
                        .build()
                        .raw_value(),
                )
                .await?;
            // Ext LNA, Int LNA, & Mixer Gain Word
            self.0
                .write_register(
                    regs::Register::GainTableWriteData1,
                    regs::gain_table_write_data1::Register::new_with_raw_value(lut_entry[0])
                        .raw_value(),
                )
                .await?;
            // TIA and LPF word.
            self.0
                .write_register(
                    regs::Register::GainTableWriteData2,
                    regs::gain_table_write_data2::Register::new_with_raw_value(lut_entry[1])
                        .raw_value(),
                )
                .await?;
            // DC Cal bit & Dig Gain Word
            self.0
                .write_register(
                    regs::Register::GainTableWriteData3,
                    regs::gain_table_write_data3::Register::new_with_raw_value(lut_entry[2])
                        .raw_value(),
                )
                .await?;
            self.0
                .write_register(
                    regs::Register::GainTableConfig,
                    regs::gain_table_config::Register::builder()
                        .with_select_rx2(true)
                        .with_select_rx1(true)
                        .with_write_gain_table(true)
                        .with_start_gain_table_clock(true)
                        .build()
                        .raw_value(),
                )
                .await?;
            // The C library uses two dummy writes to perform the four RX sample period delay.
            self.0
                .write_register(regs::Register::GainTableReadData1, 0)
                .await?;
            self.0
                .write_register(regs::Register::GainTableReadData1, 0)
                .await?;
            // For the full table case, TIA needs to be 1 and LPF needs to be 0. In the split table
            // case, we only care about TIA being 1.
            if lut_entry[1] & lpf_tia_mask == TIA_BIT_MASK {
                gain_config_for_tx_quad.0 = Some(u7::new(index as u8));
            }
        }
        // Clear write-bit.
        self.0
            .write_register(
                regs::Register::GainTableConfig,
                regs::gain_table_config::Register::builder()
                    .with_select_rx2(true)
                    .with_select_rx1(true)
                    .with_write_gain_table(false)
                    .with_start_gain_table_clock(true)
                    .build()
                    .raw_value(),
            )
            .await?;
        // The C library uses two dummy writes to perform the four RX sample period delay.
        self.0
            .write_register(regs::Register::GainTableReadData1, 0)
            .await?;
        self.0
            .write_register(regs::Register::GainTableReadData1, 0)
            .await?;
        // Stop clock as well.
        self.0
            .write_register(
                regs::Register::GainTableConfig,
                regs::gain_table_config::Register::builder()
                    .with_select_rx2(false)
                    .with_select_rx1(false)
                    .with_write_gain_table(false)
                    .with_start_gain_table_clock(false)
                    .build()
                    .raw_value(),
            )
            .await?;
        Ok(gain_config_for_tx_quad)
    }

    async fn setup_tx_pll(
        &mut self,
        lut_type: SynthLutType,
        config: &clocks::RfPllConfig,
        clocks: &Clocks,
    ) -> Result<(), InitError<Spi::Error>> {
        self.setup_rf_pll(true, lut_type, config, clocks).await
    }

    async fn setup_rx_pll(
        &mut self,
        lut_type: SynthLutType,
        config: &clocks::RfPllConfig,
        clocks: &Clocks,
    ) -> Result<(), InitError<Spi::Error>> {
        self.setup_rf_pll(false, lut_type, config, clocks).await
    }

    async fn setup_rf_pll(
        &mut self,
        tx: bool,
        lut_type: SynthLutType,
        config: &clocks::RfPllConfig,
        clocks: &Clocks,
    ) -> Result<(), InitError<Spi::Error>> {
        let (synth_ref_clk, pll_vco) = if tx {
            (
                clocks.tx().synth_ref(),
                clocks
                    .tx()
                    .pll_vco()
                    .expect("PLL VCO value is unexpectedly none"),
            )
        } else {
            (
                clocks.rx().synth_ref(),
                clocks
                    .rx()
                    .pll_vco()
                    .expect("PLL VCO value is unexpectedly none"),
            )
        };
        // TODO: The C code disabled fastlock mode or does some handling if fast
        // lock was set up beforehand.
        //
        let synth_lut = if synth_ref_clk < 50_000_000 {
            match lut_type {
                SynthLutType::Fdd => LutFrequency::_40Mhz.get_fdd_lut(),
                SynthLutType::Tdd => LutFrequency::_40Mhz.get_tdd_lut(),
            }
        } else if synth_ref_clk < 70_000_000 {
            match lut_type {
                SynthLutType::Fdd => LutFrequency::_60Mhz.get_fdd_lut(),
                SynthLutType::Tdd => LutFrequency::_60Mhz.get_tdd_lut(),
            }
        } else {
            match lut_type {
                SynthLutType::Fdd => LutFrequency::_80Mhz.get_fdd_lut(),
                SynthLutType::Tdd => LutFrequency::_80Mhz.get_tdd_lut(),
            }
        };

        let mut lut_index = 0;

        // The way the clock structure is created, PL VCO should never be none.
        let pll_vco_mhz = (pll_vco / 1_000_000) as u16;
        while lut_index < synth_lut.len() - 1 && synth_lut[lut_index].vco_mhz > pll_vco_mhz {
            lut_index += 1;
        }

        let lut_entry = synth_lut[lut_index];
        let (
            reg_vco_out,
            reg_alc_varactor,
            reg_vco_bias_1,
            reg_force_vco_tune_1,
            reg_vco_varactor_ctrl_1,
            reg_vco_varactor_ctrl_0,
            reg_vco_cal_ref,
            reg_cp_current,
            reg_loop_filter_1,
            reg_loop_filter_2,
            reg_loop_filter_3,
            start_reg_rf_synth_divs,
            reg_vco_lock,
        ) = if tx {
            (
                regs::Register::TxVcoOutput,
                regs::Register::TxAlcVaractor,
                regs::Register::TxVcoBias1,
                regs::Register::TxForceVcoTune1,
                regs::Register::TxVcoVaractorCtrl1,
                regs::Register::TxVcoVaractorCtrl0,
                regs::Register::TxVcoCalRef,
                regs::Register::TxCpCurrent,
                regs::Register::TxLoopFilter1,
                regs::Register::TxLoopFilter2,
                regs::Register::TxLoopFilter3,
                regs::Register::TxFractByte2,
                regs::Register::TxCpOverrangeVcoLock,
            )
        } else {
            (
                regs::Register::RxVcoOutput,
                regs::Register::RxAlcVaractor,
                regs::Register::RxVcoBias1,
                regs::Register::RxForceVcoTune1,
                regs::Register::RxVcoVaractorCtrl1,
                regs::Register::RxVcoVaractorCtrl0,
                regs::Register::RxVcoCalRef,
                regs::Register::RxCpCurrent,
                regs::Register::RxLoopFilter1,
                regs::Register::RxLoopFilter2,
                regs::Register::RxLoopFilter3,
                regs::Register::RxFractByte2,
                regs::Register::RxCpOverrangeVcoLock,
            )
        };

        self.0
            .write_register(
                reg_vco_out,
                regs::vco_output::Register::builder()
                    .with_porb_vco_logic(true)
                    .with_vco_output_level(lut_entry.vco_output_level)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .modify_register(reg_alc_varactor, |val| {
                let mut reg = regs::alc_varactor::Register::new_with_raw_value(val);
                reg.set_vco_varactor(lut_entry.vco_varactor);
                reg.raw_value()
            })
            .await?;
        self.0
            .write_register(
                reg_vco_bias_1,
                regs::vco_bias_1::Register::builder()
                    .with_must_be_zeros(u2::ZERO)
                    .with_vco_bias_tcf(lut_entry.vco_bias_tcf)
                    .with_vco_bias_ref(lut_entry.vco_bias_ref)
                    .build()
                    .raw_value(),
            )
            .await?;

        self.0
            .write_register(
                reg_force_vco_tune_1,
                regs::force_vco_tune_1::Register::builder()
                    .with_cal_offset(lut_entry.vco_cal_offset)
                    .with_force_enable(false)
                    .with_force_tune_upper_bit(false)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                reg_vco_varactor_ctrl_1,
                regs::vco_varactor_ctrl_1::Register::builder()
                    .with_vco_varactor_reference(lut_entry.vco_varactor_reference)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0.write_register(reg_vco_cal_ref, 0).await?;
        self.0
            .write_register(
                reg_vco_varactor_ctrl_0,
                regs::vco_varactor_ctrl_0::Register::builder()
                    .with_vco_varactor_offset(u4::ZERO)
                    .with_vco_varactor_reference_tcf(u3::new(7))
                    .build()
                    .raw_value(),
            )
            .await?;

        self.0
            .modify_register(reg_cp_current, |val| {
                let mut reg = regs::cp_current::Register::new_with_raw_value(val);
                reg.set_charge_pump_current(lut_entry.charge_pump_reg);
                reg.raw_value()
            })
            .await?;
        self.0
            .write_register(
                reg_loop_filter_1,
                regs::loop_filter_1::Register::builder()
                    .with_loop_filter_c2(lut_entry.lf_c2)
                    .with_loop_filter_c1(lut_entry.lf_c1)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                reg_loop_filter_2,
                regs::loop_filter_2::Register::builder()
                    .with_loop_filter_r1(lut_entry.lf_r1)
                    .with_loop_filter_c3(lut_entry.lf_c3)
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                reg_loop_filter_3,
                regs::loop_filter_3::Register::builder()
                    .with_loop_filter_r3(lut_entry.lf_r3)
                    .with_loop_filter_bypass_r3(false)
                    .with_loop_filter_bypass_r1(false)
                    .with_loop_filter_bypass_c2(false)
                    .with_loop_filter_bypass_c1(false)
                    .build()
                    .raw_value(),
            )
            .await?;

        let integer_byte_1_reg_raw = if tx {
            self.0.read_register(regs::Register::TxIntegerByte0).await?
        } else {
            self.0.read_register(regs::Register::RxIntegerByte0).await?
        };
        let mut tx_buf: [u8; 7] = [0; 7];
        self.0
            .write_multiple_bytes_decrementing(
                &mut tx_buf,
                start_reg_rf_synth_divs,
                |payload_buf| {
                    payload_buf[0] = ((config.pll_frac.value() >> 16) & 0xff) as u8;
                    payload_buf[1] = ((config.pll_frac.value() >> 8) & 0xff) as u8;
                    payload_buf[2] = (config.pll_frac.value() & 0xff) as u8;
                    payload_buf[3] =
                        regs::integer_byte_1::Register::new_with_raw_value(integer_byte_1_reg_raw)
                            .with_synth_integer_word(u3::new(
                                ((config.pll_int.value() >> 8) & 0xff) as u8,
                            ))
                            .raw_value();
                    payload_buf[4] = (config.pll_int.value() & 0xff) as u8;
                },
            )
            .await?;

        self.0
            .modify_register(regs::Register::RfPllDividers, |val| {
                let mut reg = regs::rfpll_dividers::Register::new_with_raw_value(val);
                if tx {
                    reg.set_tx_vco_divider(config.vco_div.as_reg_bits());
                } else {
                    reg.set_rx_vco_divider(config.vco_div.as_reg_bits());
                }
                reg.raw_value()
            })
            .await?;

        let mut elapsed = 0;
        loop {
            let reg = self.0.read_register(reg_vco_lock).await?;
            if (reg >> 1) & 0b1 == 1 {
                break;
            }
            if elapsed > 20_000 {
                return Err(InitError::CalibrationTimeout(if tx {
                    ModuleId::TxPll
                } else {
                    ModuleId::RxPll
                }));
            }
            self.0.delay.delay_us(120).await;
            elapsed += 120;
        }

        Ok(())
    }

    async fn load_mixer_subtable(&mut self) -> Result<(), Spi::Error> {
        // Start clock first.
        self.0
            .write_register(
                regs::Register::GmSubTableConfig,
                regs::gm_sub_table_config::Register::builder()
                    .with_start_gm_sub_table_clock(true)
                    .with_write_gm_sub_table(false)
                    .build()
                    .raw_value(),
            )
            .await?;
        let last_index = lut::mixer_gain::CTRL_LUT.len() - 1;
        for (index, (ctrl_lut_entry, gain_lut_entry)) in lut::mixer_gain::CTRL_LUT
            .iter()
            .zip(lut::mixer_gain::GAIN_LUT.iter())
            .enumerate()
        {
            let addr = (last_index - index) as u8;
            self.0
                .write_register(regs::Register::GmSubTableAddress, addr)
                .await?;
            self.0
                .write_register(regs::Register::GmSubTableBiasWrite, 0)
                .await?;
            self.0
                .write_register(regs::Register::GmSubTableGainWrite, *gain_lut_entry)
                .await?;
            self.0
                .write_register(regs::Register::GmSubTableCtrlWrite, *ctrl_lut_entry)
                .await?;
            self.0
                .write_register(
                    regs::Register::GmSubTableConfig,
                    regs::gm_sub_table_config::Register::builder()
                        .with_start_gm_sub_table_clock(true)
                        .with_write_gm_sub_table(true)
                        .build()
                        .raw_value(),
                )
                .await?;
            // Two dummy reads for the required delay.
            self.0
                .write_register(regs::Register::GmSubTableGainRead, 0)
                .await?;
            self.0
                .write_register(regs::Register::GmSubTableGainRead, 0)
                .await?;
        }
        // Clear write-bit
        self.0
            .write_register(
                regs::Register::GmSubTableConfig,
                regs::gm_sub_table_config::Register::builder()
                    .with_start_gm_sub_table_clock(true)
                    .with_write_gm_sub_table(false)
                    .build()
                    .raw_value(),
            )
            .await?;
        // Two dummy reads for the required delay.
        self.0
            .write_register(regs::Register::GmSubTableGainRead, 0)
            .await?;
        self.0
            .write_register(regs::Register::GmSubTableGainRead, 0)
            .await?;
        // Stop clock.
        self.0
            .write_register(
                regs::Register::GmSubTableConfig,
                regs::gm_sub_table_config::Register::builder()
                    .with_start_gm_sub_table_clock(false)
                    .with_write_gm_sub_table(false)
                    .build()
                    .raw_value(),
            )
            .await?;
        Ok(())
    }

    async fn generic_tx_rx_synth_cp_calibration(
        &mut self,
        refin_hz: u32,
        fdd: bool,
        tx: bool,
    ) -> Result<(), InitError<Spi::Error>> {
        let cal_count = match fdd {
            true => u2::new(3),
            false => {
                if refin_hz > 40_000_000 {
                    u2::new(1)
                } else {
                    u2::new(0)
                }
            }
        };
        let (
            cp_level_detect_reg,
            dsm_setup_1,
            lo_gen_power_mode,
            vco_ldo,
            vco_pd_overrides,
            cp_current,
            cp_config,
            vco_cal,
            cal_status,
        ) = match tx {
            true => (
                regs::Register::TxCpLevelDetect,
                regs::Register::TxDsmSetup1,
                regs::Register::TxLoGenPowerMode,
                regs::Register::TxVcoLdo,
                regs::Register::TxVcoPdOverrides,
                regs::Register::TxCpCurrent,
                regs::Register::TxCpConfig,
                regs::Register::TxVcoCal,
                regs::Register::TxCalStatus,
            ),
            false => (
                regs::Register::RxCpLevelDetect,
                regs::Register::RxDsmSetup1,
                regs::Register::RxLoGenPowerMode,
                regs::Register::RxVcoLdo,
                regs::Register::RxVcoPdOverrides,
                regs::Register::RxCpCurrent,
                regs::Register::RxCpConfig,
                regs::Register::RxVcoCal,
                regs::Register::RxCalStatus,
            ),
        };
        let (vco_cal_reg, disable_cp_bleed, start_calib) = match tx {
            true => (
                regs::rx_vco_cal::Register::ZERO
                    .with_vco_cal_en(true)
                    .with_vco_cal_alc_wait(u3::ZERO)
                    .with_vco_cal_count(cal_count)
                    .with_should_be_0b10(u2::new(0b10))
                    .raw_value(),
                regs::rx_cp_config::Register::ZERO
                    .with_cp_offset_off(true)
                    .raw_value(),
                regs::rx_cp_config::Register::ZERO
                    .with_cp_offset_off(true)
                    .with_cp_cal_enable(true)
                    .raw_value(),
            ),
            false => (
                regs::tx_vco_cal::Register::ZERO
                    .with_vco_cal_en(true)
                    .with_vco_cal_alc_wait(u3::ZERO)
                    .with_vco_cal_count(cal_count)
                    .with_fb_clock_adv(u2::new(0b10))
                    .raw_value(),
                regs::tx_cp_config::Register::ZERO
                    .with_cp_offset_off(true)
                    .raw_value(),
                regs::tx_cp_config::Register::ZERO
                    .with_cp_offset_off(true)
                    .with_cp_cal_enable(true)
                    .raw_value(),
            ),
        };

        self.0.write_register(cp_level_detect_reg, 0x17).await?;
        self.0.write_register(dsm_setup_1, 0x0).await?;
        self.0.write_register(lo_gen_power_mode, 0x0).await?;
        self.0.write_register(vco_ldo, 0x0B).await?;
        self.0.write_register(vco_pd_overrides, 0x02).await?;
        self.0.write_register(cp_current, 0x80).await?;
        // Setting this value disables the charge pump bleed current.
        self.0.write_register(cp_config, disable_cp_bleed).await?;

        self.0.write_register(vco_cal, vco_cal_reg).await?;

        self.0
            .write_register(
                regs::Register::EnsmConfig2,
                regs::ensm_config_2::Register::ZERO
                    .with_dual_synth_mode(true)
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::EnsmConfig1,
                regs::ensm_config_1::Register::ZERO
                    .with_force_alert_state(true)
                    .with_to_alert(true)
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::EnsmMode,
                regs::ensm_mode::Register::builder()
                    .with_fdd_mode(true)
                    .build()
                    .raw_value(),
            )
            .await?;

        self.0.write_register(cp_config, start_calib).await?;
        let mut iterations = 0u32;
        loop {
            let cal_status_raw = self.0.read_register(cal_status).await?;

            match tx {
                true => {
                    let cal_status =
                        regs::tx_cal_status::Register::new_with_raw_value(cal_status_raw);
                    if cal_status.cp_cal_done() {
                        break;
                    }
                }
                false => {
                    let cal_status =
                        regs::rx_cal_status::Register::new_with_raw_value(cal_status_raw);
                    if cal_status.cp_cal_done() {
                        break;
                    }
                }
            }
            if iterations >= 20_000 {
                return Err(InitError::CalibrationTimeout(if tx {
                    ModuleId::TxPll
                } else {
                    ModuleId::RxPll
                }));
            }
            self.0.delay.delay_us(120).await;
            iterations += 1;
        }
        Ok(())
    }

    /// Run the RF DC offset calibration and wait until it self-clears.
    async fn run_rfdc_calibration(&mut self) -> Result<(), InitError<Spi::Error>> {
        self.0
            .write_register(
                regs::Register::CalibrationCtrl,
                regs::calibration_ctrl::Register::ZERO
                    .with_rfdc_cal(true)
                    .raw_value(),
            )
            .await?;

        let mut iterations = 0u32;
        loop {
            let reg = regs::calibration_ctrl::Register::new_with_raw_value(
                self.0
                    .read_register(regs::Register::CalibrationCtrl)
                    .await?,
            );
            if !reg.rfdc_cal() {
                return Ok(());
            }
            if iterations >= 20_000 {
                return Err(InitError::CalibrationTimeout(ModuleId::RfDcOffset));
            }
            self.0.delay.delay_us(1200).await;
            iterations += 1;
        }
    }

    /// Run the BB DC offset calibration and wait until it self-clears.
    async fn run_bbdc_calibration(&mut self) -> Result<(), InitError<Spi::Error>> {
        self.0
            .write_register(
                regs::Register::CalibrationCtrl,
                regs::calibration_ctrl::Register::ZERO
                    .with_bbdc_cal(true)
                    .raw_value(),
            )
            .await?;

        let mut iterations = 0u32;
        loop {
            let reg = regs::calibration_ctrl::Register::new_with_raw_value(
                self.0
                    .read_register(regs::Register::CalibrationCtrl)
                    .await?,
            );
            if !reg.bbdc_cal() {
                return Ok(());
            }
            if iterations >= 20_000 {
                return Err(InitError::CalibrationTimeout(ModuleId::BbDc));
            }
            self.0.delay.delay_us(1200).await;
            iterations += 1;
        }
    }

    /// BB DC offset calibration.
    ///
    /// Rust port of `ad9361_bb_dc_offset_calib` in the C driver.
    async fn calibrate_bb_dc_offset(&mut self) -> Result<(), InitError<Spi::Error>> {
        self.0
            .write_register(
                regs::Register::BbDcOffsetCount,
                regs::bb_dc_offset_count::Register::ZERO
                    .with_count(u6::new(63))
                    .raw_value(),
            )
            .await?;

        self.0
            .write_register(
                regs::Register::BbDcOffsetShift,
                regs::bb_dc_offset_shift::Register::ZERO
                    .with_bb_dc_m_shift(u5::new(15))
                    .raw_value(),
            )
            .await?;

        self.0
            .write_register(
                regs::Register::BbDcOffsetAtten,
                regs::bb_dc_offset_atten::Register::ZERO
                    .with_bb_dc_offset_atten(u4::new(1))
                    .raw_value(),
            )
            .await?;

        self.run_bbdc_calibration().await
    }

    /// RF DC offset calibration.
    ///
    /// Rust port of `ad9361_rf_dc_offset_calib` in the C driver.
    async fn calibrate_rf_dc_offset(
        &mut self,
        rx_lo: u64,
        config: &config::ConfigRaw,
    ) -> Result<(), InitError<Spi::Error>> {
        self.0
            .write_register(
                regs::Register::WaitCount,
                regs::wait_count::Register::ZERO
                    .with_wait_count(u6::new(0x20))
                    .raw_value(),
            )
            .await?;

        let (count, atten) = if rx_lo <= 4_000_000_000 {
            (
                config.dc_offset_count_low_range,
                config.dc_offset_attenuation_low_range,
            )
        } else {
            (
                config.dc_offset_count_high_range,
                config.dc_offset_attenuation_high_range,
            )
        };
        let dac_fs = if rx_lo <= 4_000_000_000 {
            u2::new(2)
        } else {
            u2::new(3)
        };

        self.0
            .write_register(regs::Register::RfDcOffsetCount, count)
            .await?;
        self.0
            .write_register(
                regs::Register::RfDcOffsetConfig1,
                regs::rf_dc_offset_config_1::Register::ZERO
                    .with_dac_fs(dac_fs)
                    .with_rf_dc_calibration_count(u4::new(4))
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::RfDcOffsetAtten,
                regs::rf_dc_offset_atten::Register::ZERO
                    .with_rf_dc_offset_atten(u5::new(atten))
                    .raw_value(),
            )
            .await?;

        self.0
            .write_register(
                regs::Register::DcOffsetConfig2,
                regs::dc_offset_config2::Register::ZERO
                    .with_use_wait_counter_for_rf_dc_init_cal(true)
                    .with_dc_offset_update(u3::new(3))
                    .raw_value(),
            )
            .await?;

        let invert_rx2 = config.digital_interface.pp2_config().invert_rx2();
        if config.digital_interface.rx1rx2_phase_inversion || invert_rx2 {
            self.0
                .write_register(
                    regs::Register::InvertBits,
                    regs::invert_bits::Register::ZERO
                        .with_invert_rx1_rf_dc_cgout_word(true)
                        .raw_value(),
                )
                .await?;
        } else {
            self.0
                .write_register(
                    regs::Register::InvertBits,
                    regs::invert_bits::Register::ZERO
                        .with_invert_rx1_rf_dc_cgout_word(true)
                        .with_invert_rx2_rf_dc_cgout_word(true)
                        .raw_value(),
                )
                .await?;
        }

        self.run_rfdc_calibration().await
    }

    async fn configure_rx_synth_clock_divider(
        &mut self,
        div: regs::ClockScaler,
    ) -> Result<(), Spi::Error> {
        self.0
            .modify_register(regs::Register::RefDivideConfig1, |val| {
                let mut reg = regs::ref_divide_config_1::Register::new_with_raw_value(val);
                reg.set_rx_ref_divider_upper_bit((div.raw_value().value() >> 1) == 1);
                reg.raw_value()
            })
            .await?;
        self.0
            .modify_register(regs::Register::RefDivideConfig2, |val| {
                let mut reg = regs::ref_divide_config_2::Register::new_with_raw_value(val);
                reg.set_rx_ref_divider_lower_bit((div.raw_value().value() & 1) == 1);
                reg.raw_value()
            })
            .await?;
        Ok(())
    }

    async fn configure_tx_synth_clock_divider(
        &mut self,
        div: regs::ClockScaler,
    ) -> Result<(), Spi::Error> {
        self.0
            .modify_register(regs::Register::RefDivideConfig2, |val| {
                let mut reg = regs::ref_divide_config_2::Register::new_with_raw_value(val);
                reg.set_tx_ref_divider(div);
                reg.set_should_be_ones_1(u3::new(0b111));
                reg.set_should_be_ones_0(u2::new(0b11));
                reg.raw_value()
            })
            .await?;
        Ok(())
    }

    async fn setup_external_lna_impl(
        &mut self,
        config: &config::ExternalLnaConfig,
    ) -> Result<(), Spi::Error> {
        self.0
            .modify_register(regs::Register::ExternalLnaCtrl, |val| {
                let mut reg = regs::external_lna_ctrl::Register::new_with_raw_value(val);
                reg.set_external_lna1_ctrl(config.rx1_gpo0_control);
                reg.set_external_lna2_ctrl(config.rx2_gpo1_control);
                reg.raw_value()
            })
            .await?;

        self.0
            .write_register(
                regs::Register::ExtLnaHighGain,
                (config.gain_md_b / 500).clamp(0, u6::MAX.as_u32()) as u8,
            )
            .await?;
        self.0
            .write_register(
                regs::Register::ExtLnaLowGain,
                (config.bypass_loss_md_b / 500).clamp(0, u6::MAX.as_u32()) as u8,
            )
            .await?;
        Ok(())
    }

    async fn setup_digital_ports_impl(
        &mut self,
        config: &config::DigitalInterfaceConfig,
    ) -> Result<(), Spi::Error> {
        self.0
            .write_register(
                regs::Register::ParallelPortConf1,
                config.pp1_config().raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::ParallelPortConf2,
                config.pp2_config().raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::ParallelPortConf3,
                config.pp3_config().raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::RxClockDataDelay,
                config.rx_default_delay.raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::TxClockDataDelay,
                config.tx_default_delay.raw_value(),
            )
            .await?;
        match &config.lvds_config {
            Some(lvds) => {
                self.0
                    .write_register(regs::Register::LvdsBiasCtrl, lvds.bias_control.raw_value())
                    .await?;
                self.0
                    .write_register(
                        regs::Register::LvdsInvertCtrl1,
                        lvds.invert1_control.raw_value(),
                    )
                    .await?;
                self.0
                    .write_register(
                        regs::Register::LvdsInvertCtrl2,
                        lvds.invert2_control.raw_value(),
                    )
                    .await?;
            }
            None => {
                // Default reset value.
                self.0
                    .write_register(regs::Register::LvdsBiasCtrl, 0x03)
                    .await?;
            }
        }
        if config.rx1rx2_phase_inversion && config.pp2_config.invert_rx2() {
            self.0
                .modify_register(regs::Register::InvertBits, |val| {
                    let mut reg = regs::invert_bits::Register::new_with_raw_value(val);
                    reg.set_invert_rx2_rf_dc_cgout_word(false);
                    reg.raw_value()
                })
                .await?;
        }
        Ok(())
    }

    async fn enable_rx_path_impl(
        &mut self,
        enable_rx_1: bool,
        enable_rx_2: bool,
    ) -> Result<(), Spi::Error> {
        self.0
            .modify_register(regs::Register::RxEnableFilterCtrl, |val| {
                let mut reg = regs::rx_enable_filter_ctrl::Register::new_with_raw_value(val);
                reg.set_enable_receiver_1(enable_rx_1);
                reg.set_enable_receiver_2(enable_rx_2);
                reg.raw_value()
            })
            .await?;
        Ok(())
    }

    async fn enable_tx_path_impl(
        &mut self,
        enable_tx_1: bool,
        enable_tx_2: bool,
    ) -> Result<(), Spi::Error> {
        self.0
            .modify_register(regs::Register::TxEnableFilterCtrl, |val| {
                let mut reg = regs::tx_enable_filter_ctrl::Register::new_with_raw_value(val);
                reg.set_enable_transmitter_1(enable_tx_1);
                reg.set_enable_transmitter_2(enable_tx_2);
                reg.raw_value()
            })
            .await?;
        Ok(())
    }

    async fn set_dcxo_tune_impl(
        &mut self,
        dcxo_coarse: u6,
        dcxo_fine: u13,
    ) -> Result<(), Spi::Error> {
        self.0
            .write_register(regs::Register::DcxoCoarseTune, dcxo_coarse.value())
            .await?;
        self.0
            .write_register(
                regs::Register::DcxoFineTuneHigh,
                ((dcxo_fine.value() >> 5) & 0xFF) as u8,
            )
            .await?;
        self.0
            .write_register(
                regs::Register::DcxoFineTuneLow,
                ((dcxo_fine.value() & 0b11111) << 3) as u8,
            )
            .await?;

        Ok(())
    }

    async fn setup_auxdac_impl(
        &mut self,
        aux_dac_config: &config::AuxDacConfig,
    ) -> Result<AuxDacValuesMv, Spi::Error> {
        let value_mv_0 = if let Some(dac1_config) = &aux_dac_config.dac_config[0] {
            Some(
                self.setup_individual_auxdac(Dac::Dac1, dac1_config.default_value_m_v)
                    .await?,
            )
        } else {
            None
        };
        let value_mv_1 = if let Some(dac2_config) = &aux_dac_config.dac_config[1] {
            Some(
                self.setup_individual_auxdac(Dac::Dac2, dac2_config.default_value_m_v)
                    .await?,
            )
        } else {
            None
        };

        self.0
            .modify_register(regs::Register::AuxdacEnableCtrl, |val| {
                let mut reg = regs::auxdac_enable_ctrl::Register::new_with_raw_value(val);
                if let Some(dac1_config) = &aux_dac_config.dac_config[0] {
                    reg.set_disable_auto_tx_aux_dac_1(!dac1_config.active_in_tx);
                    reg.set_disable_auto_rx_aux_dac_1(!dac1_config.active_in_rx);
                    reg.set_disable_dac1_in_alert(!dac1_config.active_in_alert);
                } else {
                    // Mirrors the C driver: an unconfigured DAC still gets its auto/alert
                    // control bits written, explicitly disabled, rather than left at whatever
                    // was in the register before (`ad9361_auxdac_set` always runs, even for a
                    // 0 mV/disabled DAC).
                    reg.set_disable_aux_dac_1(true);
                    reg.set_disable_auto_tx_aux_dac_1(true);
                    reg.set_disable_auto_rx_aux_dac_1(true);
                    reg.set_disable_dac1_in_alert(true);
                }

                if let Some(dac2_config) = &aux_dac_config.dac_config[1] {
                    reg.set_disable_auto_tx_aux_dac_2(!dac2_config.active_in_tx);
                    reg.set_disable_auto_rx_aux_dac_2(!dac2_config.active_in_rx);
                    reg.set_disable_dac2_in_alert(!dac2_config.active_in_alert);
                } else {
                    reg.set_disable_aux_dac_2(true);
                    reg.set_disable_auto_tx_aux_dac_2(true);
                    reg.set_disable_auto_rx_aux_dac_2(true);
                    reg.set_disable_dac2_in_alert(true);
                }
                reg.raw_value()
            })
            .await?;

        self.0
            .modify_register(regs::Register::ExternalLnaCtrl, |val| {
                let mut reg = regs::external_lna_ctrl::Register::new_with_raw_value(val);
                reg.set_auxdac_manual_select(aux_dac_config.aux_dac_manual_mode_enable);
                reg.raw_value()
            })
            .await?;
        if let Some(dac1_config) = &aux_dac_config.dac_config[0] {
            self.0
                .write_register(regs::Register::Auxdac1RxDelay, dac1_config.rx_delay_us)
                .await?;
            self.0
                .write_register(regs::Register::Auxdac1TxDelay, dac1_config.tx_delay_us)
                .await?;
        }
        if let Some(dac2_config) = &aux_dac_config.dac_config[1] {
            self.0
                .write_register(regs::Register::Auxdac2RxDelay, dac2_config.rx_delay_us)
                .await?;
            self.0
                .write_register(regs::Register::Auxdac2TxDelay, dac2_config.tx_delay_us)
                .await?;
        }
        Ok(AuxDacValuesMv(value_mv_0, value_mv_1))
    }

    async fn setup_gpo_impl(&mut self, gpo_config: &config::GpoConfig) -> Result<(), Spi::Error> {
        self.0
            .write_register(
                regs::Register::AutoGpo,
                regs::auto_gpo::Register::builder()
                    .with_rx_enable([
                        gpo_config.gpo[0].slave_rx_enable,
                        gpo_config.gpo[1].slave_rx_enable,
                        gpo_config.gpo[2].slave_rx_enable,
                        gpo_config.gpo[3].slave_rx_enable,
                    ])
                    .with_tx_enable([
                        gpo_config.gpo[0].slave_tx_enable,
                        gpo_config.gpo[1].slave_tx_enable,
                        gpo_config.gpo[2].slave_tx_enable,
                        gpo_config.gpo[3].slave_tx_enable,
                    ])
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(
                regs::Register::GpoForceAndInit,
                regs::gpo_force_and_init::Register::builder()
                    .with_manual_control([
                        gpo_config.gpo[0].init_state,
                        gpo_config.gpo[1].init_state,
                        gpo_config.gpo[2].init_state,
                        gpo_config.gpo[3].init_state,
                    ])
                    .with_init_state([
                        gpo_config.gpo[0].inactive_state_high,
                        gpo_config.gpo[1].inactive_state_high,
                        gpo_config.gpo[2].inactive_state_high,
                        gpo_config.gpo[3].inactive_state_high,
                    ])
                    .build()
                    .raw_value(),
            )
            .await?;
        self.0
            .write_register(regs::Register::Gpo0RxDelay, gpo_config.gpo[0].rx_delay_us)
            .await?;
        self.0
            .write_register(regs::Register::Gpo0TxDelay, gpo_config.gpo[0].tx_delay_us)
            .await?;
        self.0
            .write_register(regs::Register::Gpo1RxDelay, gpo_config.gpo[1].rx_delay_us)
            .await?;
        self.0
            .write_register(regs::Register::Gpo1TxDelay, gpo_config.gpo[1].tx_delay_us)
            .await?;
        self.0
            .write_register(regs::Register::Gpo2RxDelay, gpo_config.gpo[2].rx_delay_us)
            .await?;
        self.0
            .write_register(regs::Register::Gpo2TxDelay, gpo_config.gpo[2].tx_delay_us)
            .await?;
        self.0
            .write_register(regs::Register::Gpo3RxDelay, gpo_config.gpo[3].rx_delay_us)
            .await?;
        self.0
            .write_register(regs::Register::Gpo3TxDelay, gpo_config.gpo[3].tx_delay_us)
            .await?;

        self.0
            .modify_register(regs::Register::ExternalLnaCtrl, |val| {
                let mut reg = regs::external_lna_ctrl::Register::new_with_raw_value(val);
                reg.set_gpo_manual_select(gpo_config.manual_mode_enable);
                reg.raw_value()
            })
            .await?;
        Ok(())
    }

    async fn setup_individual_auxdac(
        &mut self,
        dac: Dac,
        value_mv: u32,
    ) -> Result<u16, Spi::Error> {
        self.0
            .modify_register(regs::Register::AuxdacEnableCtrl, |val| {
                let mut reg = regs::auxdac_enable_ctrl::Register::new_with_raw_value(val);
                match dac {
                    Dac::Dac1 => reg.set_disable_aux_dac_1(false),
                    Dac::Dac2 => reg.set_disable_aux_dac_2(false),
                }
                reg.raw_value()
            })
            .await?;

        let mut val;
        let val_mv = value_mv.max(306);

        let vref = if val_mv < 1888 {
            // Vref = 1V, Step = 2
            val = ((val_mv - 306) * 1000) / 1469;
            regs::auxdac_config::Vref::_1V
        } else {
            // Vref = 2.5V, Step = 2
            val = ((val_mv - 1761) * 1000) / 1512;
            regs::auxdac_config::Vref::_2_5V
        };
        val = val.clamp(0, 1023);

        let config_reg = regs::auxdac_config::Register::builder()
            .with_must_be_zero(false)
            .with_step_factor(false)
            .with_vref(vref)
            .with_word_lower_2_bits(u2::new((val & 0b11) as u8))
            .build();
        match dac {
            Dac::Dac1 => {
                self.0
                    .write_register(regs::Register::Auxdac1Word, (val >> 2) as u8)
                    .await?;
                self.0
                    .write_register(regs::Register::Auxdac1Config, config_reg.raw_value())
                    .await?;
            }
            Dac::Dac2 => {
                self.0
                    .write_register(regs::Register::Auxdac2Word, (val >> 2) as u8)
                    .await?;
                self.0
                    .write_register(regs::Register::Auxdac2Config, config_reg.raw_value())
                    .await?;
            }
        }
        Ok(val_mv as u16)
    }
}

struct Ad9361Core<Spi: SpiDevice, ResetPin: OutputPin, Delay: DelayNs> {
    spi: core::cell::RefCell<Spi>,
    reset_pin: ResetPin,
    delay: Delay,
}

#[bisync]
impl<Spi: SpiDevice, ResetPin: OutputPin, Delay: DelayNs> Ad9361Core<Spi, ResetPin, Delay> {
    fn new(spi: Spi, reset_pin: ResetPin, delay: Delay) -> Self {
        Self {
            spi: core::cell::RefCell::new(spi),
            reset_pin,
            delay,
        }
    }

    pub async fn reset(&mut self) -> Result<(), ResetError<Spi::Error, ResetPin::Error>> {
        self.reset_pin.set_low().map_err(ResetError::Gpio)?;
        self.delay.delay_ms(1).await;
        self.reset_pin.set_high().map_err(ResetError::Gpio)?;
        self.delay.delay_ms(1).await;

        // C lib mentions that this does not work reliably without full HW pin reset.
        self.write_register(
            regs::Register::SpiConf,
            regs::spi_conf::Register::default()
                .with_soft_reset(true)
                .with_soft_reset_mirror(true)
                .raw_value(),
        )
        .await
        .map_err(ResetError::Spi)?;
        self.write_register(regs::Register::SpiConf, 0)
            .await
            .map_err(ResetError::Spi)?;
        Ok(())
    }

    pub async fn modify_register<F: Fn(u8) -> u8>(
        &mut self,
        reg: regs::Register,
        f: F,
    ) -> Result<(), Spi::Error> {
        let current_value = self.read_register(reg).await?;
        let new_value = f(current_value);
        self.write_register(reg, new_value).await
    }

    /// Write multiple bytes starting from the specified register, with the register address
    /// automatically decrementing.
    ///
    /// This function will directly return if the provided buffer is smaller than 2. Otherwise,
    /// it prepares the first 2 bytes of the passed buffer for the transfer, while passing the
    /// remaining buffer to the user-provided closure as the payload buffer.
    pub async fn write_multiple_bytes_decrementing<F: FnOnce(&mut [u8])>(
        &mut self,
        buf: &mut [u8],
        start_reg: regs::Register,
        f: F,
    ) -> Result<(), Spi::Error> {
        if buf.len() < 2 {
            return Ok(());
        }
        spi::prepare_spi_write(buf, start_reg.as_u10(), u3::new((buf.len() - 2) as u8));
        f(&mut buf[2..]);
        self.spi.get_mut().write(buf).await?;
        Ok(())
    }

    pub async fn write_register(&mut self, reg: regs::Register, val: u8) -> Result<(), Spi::Error> {
        let mut buf: [u8; 3] = [0; 3];
        spi::prepare_spi_write(&mut buf, reg.as_u10(), u3::new(1));
        buf[2] = val;
        self.spi.get_mut().write(&buf).await?;
        Ok(())
    }

    pub async fn write_register_raw(&mut self, reg: u10, val: u8) -> Result<(), Spi::Error> {
        let mut buf: [u8; 3] = [0; 3];
        spi::prepare_spi_write(&mut buf, reg, u3::new(1));
        buf[2] = val;
        self.spi.get_mut().write(&buf).await?;
        Ok(())
    }

    pub async fn read_register(&self, reg: regs::Register) -> Result<u8, Spi::Error> {
        self.read_register_raw(reg.as_u10()).await
    }

    #[allow(clippy::await_holding_refcell_ref)]
    pub async fn read_register_raw(&self, reg: u10) -> Result<u8, Spi::Error> {
        let mut buf: [u8; 3] = [0; 3];
        spi::prepare_spi_read(&mut buf, reg, u3::new(1));
        self.spi.borrow_mut().transfer_in_place(&mut buf).await?;
        Ok(buf[2])
    }

    /*
    pub async fn set_ensm_state(
        &mut self,
        current: EnsmState,
        target: EnsmState,
        external_clock_config: config::ExternalClockConfig,
    ) -> Result<(), Spi::Error> {
        // TODO: implementation.
        if current == EnsmState::Sleep {
            self.write_register(
                regs::Register::ClockEnable,
                regs::clock_enable::Register::builder()
                    .with_xo_bypass(external_clock_config.into())
                    .with_clock_enable_dflt(true)
                    .with_digital_power_up(true)
                    .with_bbpll_enable(true)
                    .build()
                    .raw_value(),
            )
            .await?;
        }
        Ok(())
    }
    */

    pub async fn read_product_id(&self) -> Result<ProductIdReg, Spi::Error> {
        self.read_register(regs::Register::ProductId)
            .await
            .map(ProductIdReg::new_with_raw_value)
    }

    /// RX BB analog filter calibration.
    ///
    /// Returns the RX BB filter divide value that was used for the calibration.
    async fn calibrate_rx_bb_analog_filter(
        &mut self,
        rx_bb_bw: u32,
        bbpll_freq: u32,
    ) -> Result<u32, InitError<Spi::Error>> {
        let rx_bb_bw = rx_bb_bw.clamp(200_000, 28_000_000);

        let target = 126_906u64 * (rx_bb_bw as u64 / 10_000);
        let rxbbf_div = (bbpll_freq as u64).div_ceil(target).min(511) as u32;

        self.write_register(regs::Register::RxBbfTuneDivide, (rxbbf_div & 0xFF) as u8)
            .await?;

        self.modify_register(regs::Register::RxBbfTuneConfig, |val| {
            let mut reg = regs::rx_bbf_tune_config::Register::new_with_raw_value(val);
            reg.set_rx_bbf_tune_divide_msb((rxbbf_div >> 8) != 0);
            reg.raw_value()
        })
        .await?;

        self.write_register(
            regs::Register::RxBbbwMhz,
            regs::rx_bbbw_mhz::Register::ZERO
                .with_rx_tune_bbbw_mhz(u5::new((rx_bb_bw / 1_000_000) as u8))
                .raw_value(),
        )
        .await?;

        let tmp = div_round_u32((rx_bb_bw % 1_000_000) * 128, 1_000_000);
        self.write_register(
            regs::Register::RxBbbwKhz,
            regs::rx_bbbw_khz::Register::ZERO
                .with_rx_tune_bbbw_khz(u7::new((tmp.min(127)) as u8))
                .raw_value(),
        )
        .await?;

        self.write_register(
            regs::Register::RxMixLoCm,
            regs::rx_mix_lo_cm::Register::ZERO
                .with_rx_mix_lo_cm(u6::new(0x3F))
                .raw_value(),
        )
        .await?;

        self.write_register(
            regs::Register::RxMixGmConfig,
            regs::rx_mix_gm_config::Register::ZERO
                .with_rx_mix_gm_pload(u2::new(3))
                .raw_value(),
        )
        .await?;

        self.write_register(
            regs::Register::Rx1TuneCtrl,
            regs::rx1_tune_ctrl::Register::ZERO
                .with_rx1_tune_resample(true)
                .raw_value(),
        )
        .await?;

        self.write_register(
            regs::Register::Rx2TuneCtrl,
            regs::rx2_tune_ctrl::Register::ZERO
                .with_rx2_tune_resample(true)
                .raw_value(),
        )
        .await?;

        self.run_rx_bb_tune_calibration().await?;

        self.write_register(
            regs::Register::Rx1TuneCtrl,
            regs::rx1_tune_ctrl::Register::ZERO
                .with_rx1_tune_resample(true)
                .with_rx1_pd_tune(true)
                .raw_value(),
        )
        .await?;

        self.write_register(
            regs::Register::Rx2TuneCtrl,
            regs::rx2_tune_ctrl::Register::ZERO
                .with_rx2_tune_resample(true)
                .with_rx2_pd_tune(true)
                .raw_value(),
        )
        .await?;

        Ok(rxbbf_div)
    }

    /// RX ADC setup.
    async fn setup_rx_adc(
        &mut self,
        bbpll_freq: u32,
        adc_sampl_freq_hz: u32,
        rxbbf_div: u32,
    ) -> Result<(), InitError<Spi::Error>> {
        let c3_msb = self.read_register(regs::Register::RxBbfC3Msb).await?;
        let c3_lsb = self.read_register(regs::Register::RxBbfC3Lsb).await?;
        let r2346 = self.read_register(regs::Register::RxBbfR2346).await?;

        let mut tmp = (bbpll_freq as u64) * 10_000u64;
        tmp /= 126_906u64 * (rxbbf_div as u64);
        let mut bb_bw_hz = tmp as u32;

        bb_bw_hz = bb_bw_hz.clamp(200_000, 28_000_000);

        let scale_snr_1e3 = if adc_sampl_freq_hz < 80_000_000 {
            1000
        } else {
            1585
        };

        let invrc_tconst_1e6 = if bb_bw_hz >= 18_000_000 {
            let factor = 1000 + (10 * (bb_bw_hz - 18_000_000)) / 1_000_000;
            let mut val = 160_975u64
                * (r2346 as u64)
                * (160 * (c3_msb as u64) + 10 * (c3_lsb as u64) + 140)
                * (bb_bw_hz as u64)
                * (factor as u64);
            val /= 1000;
            val
        } else {
            160_975u64
                * (r2346 as u64)
                * (160 * (c3_msb as u64) + 10 * (c3_lsb as u64) + 140)
                * (bb_bw_hz as u64)
        };
        let invrc_tconst_1e6 = invrc_tconst_1e6 / 1_000_000_000;

        let sqrt_inv_rc_tconst_1e3 = num::integer::sqrt(invrc_tconst_1e6 as u32);
        let maxsnr = 640 / 160; // = 4
        let scaled_adc_clk_1e6 = div_round_u32(adc_sampl_freq_hz, 640);

        let inv_scaled_adc_clk_1e3 =
            div_round_u32(640_000_000, div_round_u32(adc_sampl_freq_hz, 1000));
        let tmp_1e3 = div_round_u32(
            980_000 + 20 * div_round_u32(inv_scaled_adc_clk_1e3, maxsnr).max(1000),
            1000,
        );
        let sqrt_term_1e3 = num::integer::sqrt(scaled_adc_clk_1e6);
        let min_sqrt_term_1e3 = num::integer::sqrt(maxsnr * scaled_adc_clk_1e6).min(1000);

        let mut data = [0u8; 40];

        data[0] = 0;
        data[1] = 0;
        data[2] = 0;
        data[3] = 0x24;
        data[4] = 0x24;
        data[5] = 0;
        data[6] = 0;

        let tmp = (8u64
            * scale_snr_1e3 as u64
            * sqrt_inv_rc_tconst_1e3 as u64
            * min_sqrt_term_1e3 as u64)
            .wrapping_sub(50_000_000);
        let tmp = tmp / 100_000_000;
        data[7] = tmp.min(124) as u8;

        let tmp = (invrc_tconst_1e6 >> 1)
            + ((((20u64 * inv_scaled_adc_clk_1e3 as u64) * data[7] as u64) / 80) * 1000);
        let tmp = tmp / invrc_tconst_1e6;
        data[8] = tmp.min(255) as u8;

        let tmp = (77u64 * sqrt_inv_rc_tconst_1e3 as u64 * min_sqrt_term_1e3 as u64)
            .wrapping_sub(500_000);
        let tmp = tmp / 1_000_000;
        data[10] = tmp.min(127) as u8;

        data[9] = ((800 * data[10] as u32) / 1000).min(127) as u8;

        let tmp = (invrc_tconst_1e6 >> 1)
            + (20u64 * inv_scaled_adc_clk_1e3 as u64 * data[10] as u64 * 1000);
        let tmp = tmp / (invrc_tconst_1e6 * 77);
        data[11] = tmp.min(255) as u8;

        let product = 80u32
            .wrapping_mul(sqrt_inv_rc_tconst_1e3)
            .wrapping_mul(min_sqrt_term_1e3);
        let tmp = product.wrapping_sub(500_000);
        let tmp = (tmp as u64) / 1_000_000;
        data[12] = tmp.min(127) as u8;

        let term1 = 3u64 * (invrc_tconst_1e6 >> 1);
        let term2 = (inv_scaled_adc_clk_1e3 as u64) * (data[12] as u64) * 250u64;
        let tmp = term2.wrapping_sub(term1);
        let tmp = tmp / invrc_tconst_1e6;
        data[13] = tmp.min(255) as u8;

        data[14] = (21 * (inv_scaled_adc_clk_1e3 / 10_000)) as u8;

        data[15] = ((500 + 1025 * data[7] as u32) / 1000).min(127) as u8;
        data[16] = ((data[15] as u32 * tmp_1e3) / 1000).min(127) as u8;
        data[17] = data[15];

        data[18] = ((500 + 975 * data[10] as u32) / 1000).min(127) as u8;
        data[19] = ((data[18] as u32 * tmp_1e3) / 1000).min(127) as u8;
        data[20] = data[18];

        data[21] = ((500 + 975 * data[12] as u32) / 1000).min(127) as u8;
        data[22] = ((data[21] as u32 * tmp_1e3) / 1000).min(127) as u8;
        data[23] = data[21];

        data[24] = 0x2E;

        let inner = div_round_u32(63 * scaled_adc_clk_1e6, 1000).min(63_000);
        data[25] = (128 + inner / 1000) as u8;

        let inner = 920 + (80 * inv_scaled_adc_clk_1e3) / 1000;
        let val = (63 * scaled_adc_clk_1e6) / 1_000_000;
        let val = val * inner / 1000;
        data[26] = val.min(63) as u8;

        data[27] = ((32 * sqrt_term_1e3) / 1000).min(63) as u8;

        data[28] = data[25];
        data[29] = data[26];
        data[30] = data[27];
        data[31] = data[25];
        data[32] = data[26];

        data[33] = ((63 * sqrt_term_1e3) / 1000).min(63) as u8;
        data[34] = ((64 * sqrt_term_1e3) / 1000).min(127) as u8;

        data[35] = 0x40;
        data[36] = 0x40;
        data[37] = 0x2C;
        data[38] = 0x00;
        data[39] = 0x00;

        for (i, &val) in data.iter().enumerate() {
            let reg = match i as u8 {
                0 => regs::Register::AdcSetup0,
                1 => regs::Register::FbDacClkDelay1,
                2 => regs::Register::FbDacClkDelay2,
                3 => regs::Register::FlashSampleClkDelay3p,
                4 => regs::Register::FlashSampleClkDelay3n,
                5 => regs::Register::TestMux2i,
                6 => regs::Register::TestMux2q,
                7 => regs::Register::Integrator1Resistance,
                8 => regs::Register::Integrator1Capacitance,
                9 => regs::Register::Integrator23Resistance,
                10 => regs::Register::Integrator2Resistance,
                11 => regs::Register::Integrator2Capacitance,
                12 => regs::Register::Integrator3Resistance,
                13 => regs::Register::Integrator3Capacitance,
                14 => regs::Register::IntegratorAmpCc,
                15 => regs::Register::Int1FbDacNmosCurrentSource,
                16 => regs::Register::Int1FbDacNmosCasoadeBiasCurrent,
                17 => regs::Register::Int1FbDacPmosCurrentSource,
                18 => regs::Register::Int2FbDacNmosCurrentSource,
                19 => regs::Register::Int2FbDacNmosCascodeBiasCurrent,
                20 => regs::Register::Int2FbDacPmosCurrentSource,
                21 => regs::Register::Int3FbDacNmosCurrentSource,
                22 => regs::Register::Int3FbDacNmosCascodeBiasCurrent,
                23 => regs::Register::Int3FbDacPmosCurrentSource,
                24 => regs::Register::FbDacBiasCurrent,
                25 => regs::Register::Int11stStageCurrent,
                26 => regs::Register::Int11stStageCascodeCurrent,
                27 => regs::Register::Int12ndStageCurrent,
                28 => regs::Register::Integrator21stStageCurrent,
                29 => regs::Register::Int21stStageCascodeCurrent,
                30 => regs::Register::Int22ndStageCurrent,
                31 => regs::Register::Int31stStageCurrent,
                32 => regs::Register::Int31stStageCascodeCurrent,
                33 => regs::Register::Int32ndStageCurrent,
                34 => regs::Register::FlashBiasCurrent,
                35 => regs::Register::FlashLadderBias,
                36 => regs::Register::FlashLadderCascodeCurrent,
                37 => regs::Register::FlashLadderBias2,
                38 => regs::Register::Reset,
                39 => regs::Register::AdcSetup39,
                _ => unreachable!(),
            };
            self.write_register(reg, val).await?;
        }

        Ok(())
    }

    /// TX BB analog filter calibration.
    ///
    /// Rust port of `ad9361_tx_bb_analog_filter_calib` in the C driver.
    async fn calibrate_tx_bb_analog_filter(
        &mut self,
        tx_bb_bw: u32,
        bbpll_freq: u32,
    ) -> Result<u16, InitError<Spi::Error>> {
        let tx_bb_bw = tx_bb_bw.clamp(625_000, 20_000_000);

        let target = 145_036u64 * (tx_bb_bw as u64 / 10_000);
        let txbbf_div = (bbpll_freq as u64).div_ceil(target).min(511) as u16;

        self.write_register(regs::Register::TxBbfTuneDivider, (txbbf_div & 0xFF) as u8)
            .await?;

        self.modify_register(regs::Register::TxBbfTuneMode, |val| {
            let mut reg = regs::tx_bbf_tune_mode::Register::new_with_raw_value(val);
            reg.set_tx_bbf_tune_divider_msb((txbbf_div >> 8) != 0);
            reg.raw_value()
        })
        .await?;

        self.write_register(
            regs::Register::TxTuneCtrl,
            regs::tx_tune_ctrl::Register::ZERO
                .with_tuner_resample(true)
                .with_tune_ctrl(u2::new(1))
                .raw_value(),
        )
        .await?;

        self.run_tx_bb_tune_calibration().await?;

        self.write_register(
            regs::Register::TxTuneCtrl,
            regs::tx_tune_ctrl::Register::ZERO
                .with_tuner_resample(true)
                .with_pd_tune(true)
                .with_tune_ctrl(u2::new(1))
                .raw_value(),
        )
        .await?;

        Ok(txbbf_div)
    }

    /// Run the RX BB tune calibration and wait until it self-clears.
    async fn run_rx_bb_tune_calibration(&mut self) -> Result<(), InitError<Spi::Error>> {
        self.write_register(
            regs::Register::CalibrationCtrl,
            regs::calibration_ctrl::Register::ZERO
                .with_rx_bb_tune_cal(true)
                .raw_value(),
        )
        .await?;

        let mut iterations = 0u32;
        loop {
            let reg = regs::calibration_ctrl::Register::new_with_raw_value(
                self.read_register(regs::Register::CalibrationCtrl).await?,
            );
            if !reg.rx_bb_tune_cal() {
                return Ok(());
            }
            if iterations >= 20_000 {
                return Err(InitError::CalibrationTimeout(ModuleId::RxBbTune));
            }
            self.delay.delay_us(1200).await;
            iterations += 1;
        }
    }

    /// Run the TX BB tune calibration and wait until it self-clears.
    async fn run_tx_bb_tune_calibration(&mut self) -> Result<(), InitError<Spi::Error>> {
        self.write_register(
            regs::Register::CalibrationCtrl,
            regs::calibration_ctrl::Register::ZERO
                .with_tx_bb_tune_cal(true)
                .raw_value(),
        )
        .await?;

        let mut iterations = 0u32;
        loop {
            let reg = regs::calibration_ctrl::Register::new_with_raw_value(
                self.read_register(regs::Register::CalibrationCtrl).await?,
            );
            if !reg.tx_bb_tune_cal() {
                return Ok(());
            }
            if iterations >= 20_000 {
                return Err(InitError::CalibrationTimeout(ModuleId::TxBbTune));
            }
            self.delay.delay_us(1200).await;
            iterations += 1;
        }
    }

    /// RX TIA calibration.
    ///
    /// Rust port of `ad9361_rx_tia_calib` in the C driver.
    async fn calibrate_rx_tia(&mut self, bb_bw_hz: u32) -> Result<(), InitError<Spi::Error>> {
        let reg1eb = self.read_register(regs::Register::RxBbfC3Msb).await?;
        let reg1ec = self.read_register(regs::Register::RxBbfC3Lsb).await?;
        let reg1e6 = self.read_register(regs::Register::RxBbfR2346).await?;

        let bb_bw_hz = bb_bw_hz.clamp(200_000, 20_000_000);

        let cbbf = (reg1eb as u64 * 160) + (reg1ec as u64 * 10) + 140;
        let r2346 = 18300u64 * (reg1e6 & 0x07) as u64;

        let ctia_ff = cbbf * r2346 * 560 / 3_500_000;

        let reg1db = if bb_bw_hz <= 3_000_000 {
            0xE0
        } else if bb_bw_hz <= 10_000_000 {
            0x60
        } else {
            0x20
        };

        let (reg1dc, reg1de, reg1dd, reg1df) = if ctia_ff > 2920 {
            let temp = div_round_u32((ctia_ff - 400) as u32, 320).min(127) as u8;
            (0x40, 0x40, temp, temp)
        } else {
            let temp =
                (div_round_u32(ctia_ff.saturating_sub(400) as u32, 40) + 0x40).min(255) as u8;
            (temp, temp, 0, 0)
        };

        self.write_register(regs::Register::RxTiaConfig, reg1db)
            .await?;
        self.write_register(regs::Register::Tia1CLsb, reg1dc)
            .await?;
        self.write_register(regs::Register::Tia1CMsb, reg1dd)
            .await?;
        self.write_register(regs::Register::Tia2CLsb, reg1de)
            .await?;
        self.write_register(regs::Register::Tia2CMsb, reg1df)
            .await?;

        Ok(())
    }

    /// TX BB second filter calibration.
    ///
    /// Rust port of `ad9361_tx_bb_second_filter_calib` in the C driver.
    async fn calibrate_tx_bb_second_analog_filter(
        &mut self,
        tx_bb_bw: u32,
    ) -> Result<(), InitError<Spi::Error>> {
        let tx_bb_bw = tx_bb_bw.clamp(530_000, 20_000_000);

        let corner = 15_708u64 * (tx_bb_bw as u64 / 10_000);

        let mut cap: u64 = 0;
        let mut res = 1u32;
        for _ in 0..4 {
            let div = corner * (res as u64);
            cap = 500_000_000u64 + (div >> 1);
            cap /= div;
            cap = cap.saturating_sub(12);
            if cap < 64 {
                break;
            }
            res <<= 1;
        }

        let cap = cap.min(63) as u8;

        let reg_conf = if tx_bb_bw <= 4_500_000 {
            0x59
        } else if tx_bb_bw <= 12_000_000 {
            0x56
        } else {
            0x57
        };

        let reg_res = match res {
            1 => 0x0C,
            2 => 0x04,
            4 => 0x03,
            8 => 0x01,
            _ => 0x01,
        };

        self.write_register(regs::Register::Config0, reg_conf)
            .await?;
        self.write_register(regs::Register::Resistor, reg_res)
            .await?;
        self.write_register(regs::Register::Capacitor, cap).await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn perform_tx_quad_calibration(
        &mut self,
        tx_channel_config: TransceiverChannelMode,
        clocks: &clocks::Clocks,
        rf_tx_bw_hz: u32,
        rf_rx_bw_hz: u32,
        rx_phase_config: RxPhaseConfig,
        gain_index_tx_quad: GainTableIndexForTxQuad,
        phase_inversion_en: bool,
    ) -> Result<(), InitError<Spi::Error>> {
        // Keep this API on RF bandwidth and convert locally for equations that use BB/2.
        let real_tx_bw_hz = rf_tx_bw_hz / 2;
        let real_rx_bw_hz = rf_rx_bw_hz / 2;

        // Find NCO frequency that matches this equation:
        // BW / 4 = Rx NCO freq = Tx NCO freq:
        // Rx NCO = ClkRF * (rxNCO <1:0> + 1) / 32
        // Tx NCO = ClkTF * (txNCO <1:0> + 1) / 32
        let clk_rx_fir = clocks.rx().rx_fir_input();
        let clk_tx_fir = clocks.tx().tx_fir();

        let mut tx_nco_word = u2::new(
            div_round_u32(real_tx_bw_hz * 8, clk_tx_fir)
                .saturating_sub(1)
                .min(3) as u8,
        );
        let mut rx_nco_word = tx_nco_word;
        let decim = u2::new(if clk_tx_fir <= 4_000_000 { 2 } else { 3 });

        let calculated_rx_phase = u5::new(if clk_rx_fir.abs_diff(clk_tx_fir * 2) < 2 {
            let mut rx_phase = 0x0E;
            match tx_nco_word.value() {
                0 => tx_nco_word = u2::new(1),
                1 => rx_nco_word = u2::new(0),
                2 => {
                    rx_nco_word = u2::new(0);
                    tx_nco_word = u2::new(1);
                }
                3 => {
                    rx_nco_word = u2::new(1);
                    rx_phase = 0x08;
                }
                _ => unreachable!(),
            }
            rx_phase
        } else if clk_rx_fir == clk_tx_fir {
            match tx_nco_word.value() {
                0 | 3 => 0x15,
                2 => 0x1F,
                1 => {
                    let filter_control = regs::tx_enable_filter_ctrl::Register::new_with_raw_value(
                        self.read_register(regs::Register::TxEnableFilterCtrl)
                            .await?,
                    );
                    if filter_control.thb3() == Ok(clocks::Thb3Interpolation::Mult3Filter)
                        && filter_control.tx_fir() == clocks::TxFirInterpolation::Mult2EnableFilter
                    {
                        0x15
                    } else {
                        0x1A
                    }
                }
                _ => unreachable!(),
            }
        } else {
            return Err(InitError::UnsymetricFirClocks {
                tx_fir_output_hz: clk_tx_fir,
                rx_fir_input_hz: clk_rx_fir,
            });
        });

        let tx_nco_freq = clk_tx_fir * (tx_nco_word.value().as_u32() + 1) / 32;
        if tx_nco_freq > (real_tx_bw_hz / 4) || tx_nco_freq > (real_rx_bw_hz / 4) {
            // This ensures the bandwidth during calibration is wide enough.
            self.update_rf_bandwidth(clocks, tx_nco_freq * 8, tx_nco_freq * 8)
                .await?;
        }

        // Mirrors ad9361_tx_quad_calib's `phase_inversion_en` handling: temporarily un-invert
        // RX2 on the digital interface and force both RX1/RX2 RF-DC-CGOUT-word inversion bits
        // while the calibration runs. An inverted RX2 data path would otherwise scramble what
        // the calibration engine observes while measuring TX LO leakage/sideband imbalance via
        // loopback, so this only matters when RX2 inversion is actually configured.
        let saved_invert_bits = if phase_inversion_en {
            self.modify_register(regs::Register::ParallelPortConf2, |val| {
                regs::parallel_port_conf_2::Register::new_with_raw_value(val)
                    .with_invert_rx2(false)
                    .raw_value()
            })
            .await?;
            let saved = self.read_register(regs::Register::InvertBits).await?;
            self.write_register(
                regs::Register::InvertBits,
                regs::invert_bits::Register::ZERO
                    .with_invert_rx1_rf_dc_cgout_word(true)
                    .with_invert_rx2_rf_dc_cgout_word(true)
                    .raw_value(),
            )
            .await?;
            Some(saved)
        } else {
            None
        };

        self.modify_register(regs::Register::Kexp2, |val| {
            let mut reg = regs::kexp_2::Register::new_with_raw_value(val);
            reg.set_tx_nco_freq(tx_nco_word);
            reg.raw_value()
        })
        .await?;
        self.write_register(regs::Register::QuadCalCount, 0xff)
            .await?;
        self.write_register(
            regs::Register::Kexp1,
            regs::kexp_1::Register::builder()
                .with_kexp_tx(u2::new(0b01))
                .with_kexp_tx_comp(u2::new(0b11))
                .with_kexp_dc_i(u2::new(0b11))
                .with_kexp_dc_q(u2::new(0b11))
                .build()
                .raw_value(),
        )
        .await?;
        self.write_register(regs::Register::MagFtestThresh, 0x03)
            .await?;
        self.write_register(regs::Register::MagFtestThresh2, 0x03)
            .await?;

        if let Some(gain_index) = gain_index_tx_quad.0 {
            self.write_register(regs::Register::TxQuadFullLmtGain, gain_index.value())
                .await?;
        }

        self.write_register(regs::Register::QuadSettleCount, 0xF0)
            .await?;
        self.write_register(regs::Register::TxQuadLpfGain, 0x00)
            .await?;

        let initial_phase = match rx_phase_config {
            RxPhaseConfig::Manual(phase) => Some(phase),
            RxPhaseConfig::Calculated => Some(calculated_rx_phase),
            RxPhaseConfig::ForceSearch => None,
        };

        let mut converged = false;
        if let Some(phase) = initial_phase {
            converged = self
                .tx_quad_calibration_inner(phase, rx_nco_word, decim, tx_channel_config)
                .await?;
        }
        if !converged {
            self.tx_quad_phase_search(rx_nco_word, decim, tx_channel_config)
                .await?;
        }

        // Cleanup must run regardless of which path above converged — restoring these is not
        // optional, so this can no longer be skipped by an early return.
        if let Some(saved) = saved_invert_bits {
            self.modify_register(regs::Register::ParallelPortConf2, |val| {
                regs::parallel_port_conf_2::Register::new_with_raw_value(val)
                    .with_invert_rx2(true)
                    .raw_value()
            })
            .await?;
            self.write_register(regs::Register::InvertBits, saved)
                .await?;
        }

        if tx_nco_freq > (real_tx_bw_hz / 4) || tx_nco_freq > (real_rx_bw_hz / 4) {
            // This ensures the bandwidth during calibration is wide enough.
            self.update_rf_bandwidth(clocks, rf_rx_bw_hz, rf_tx_bw_hz)
                .await?;
        }
        Ok(())
    }

    async fn tx_quad_phase_search(
        &mut self,
        rx_nco_word: u2,
        decimation: u2,
        tx_channel_config: TransceiverChannelMode,
    ) -> Result<u5, InitError<Spi::Error>> {
        // The C implementation duplicates 32 phase results into a 64-entry field so an optimal
        // window that crosses 31->0 is represented as one contiguous segment.
        let mut failed: [bool; 64] = [true; 64];

        for phase in 0..32u8 {
            let converged = self
                .tx_quad_calibration_inner(
                    u5::new(phase),
                    rx_nco_word,
                    decimation,
                    tx_channel_config,
                )
                .await?;
            let did_fail = !converged;
            failed[phase as usize] = did_fail;
            failed[phase as usize + 32] = did_fail;
        }

        let (window_start, window_len) = find_longest_success_window(&failed);
        let best_phase = u5::new(((window_start + window_len / 2) & 0x1f) as u8);

        let _ = self
            .tx_quad_calibration_inner(best_phase, rx_nco_word, decimation, tx_channel_config)
            .await?;

        Ok(best_phase)
    }

    async fn tx_quad_calibration_inner(
        &mut self,
        rx_phase: u5,
        rx_nco_word: u2,
        decimation: u2,
        tx_channel_config: TransceiverChannelMode,
    ) -> Result<bool, InitError<Spi::Error>> {
        self.write_register(
            regs::Register::QuadCalNcoFreqPhaseOffset,
            regs::quad_cal_nco_freq_phase_offset::Register::builder()
                .with_rx_nco_freq(rx_nco_word)
                .with_rx_nco_phase_offset(rx_phase)
                .build()
                .raw_value(),
        )
        .await?;
        self.write_register(
            regs::Register::QuadCalCtrl,
            regs::quad_cal_ctrl::Register::builder()
                .with_free_run_enable(false)
                .with_settle_main_enable(true)
                .with_dc_offset_enable(true)
                .with_quad_cal_soft_reset(true)
                .with_gain_enable(true)
                .with_phase_enable(true)
                .with_m_decim(decimation)
                .build()
                .raw_value(),
        )
        .await?;
        self.write_register(
            regs::Register::QuadCalCtrl,
            regs::quad_cal_ctrl::Register::builder()
                .with_free_run_enable(false)
                .with_settle_main_enable(true)
                .with_dc_offset_enable(true)
                .with_quad_cal_soft_reset(false)
                .with_gain_enable(true)
                .with_phase_enable(true)
                .with_m_decim(decimation)
                .build()
                .raw_value(),
        )
        .await?;

        self.write_register(
            regs::Register::CalibrationCtrl,
            regs::calibration_ctrl::Register::ZERO
                .with_tx_quad_cal(true)
                .raw_value(),
        )
        .await?;

        let mut iterations = 0u32;
        loop {
            let reg = regs::calibration_ctrl::Register::new_with_raw_value(
                self.read_register(regs::Register::CalibrationCtrl).await?,
            );
            if !reg.tx_quad_cal() {
                break;
            }
            if iterations >= 20_000 {
                return Err(InitError::CalibrationTimeout(ModuleId::TxQuad));
            }
            self.delay.delay_us(1200).await;
            iterations += 1;
        }

        let tx1 = regs::quad_cal_status_tx::Register::new_with_raw_value(
            self.read_register(regs::Register::QuadCalStatusTx1).await?,
        );
        let tx2 = regs::quad_cal_status_tx::Register::new_with_raw_value(
            self.read_register(regs::Register::QuadCalStatusTx2).await?,
        );
        let tx1_converged = tx1.tx_lo_conv() && tx1.tx_ssb_conv();
        let tx2_converged = tx2.tx_lo_conv() && tx2.tx_ssb_conv();

        Ok(match tx_channel_config {
            TransceiverChannelMode::Dual => tx1_converged && tx2_converged,
            TransceiverChannelMode::Single { rx: _, tx } => match tx {
                TransmitterId::Tx1 => tx1_converged,
                TransmitterId::Tx2 => tx2_converged,
            },
        })
    }

    async fn update_rf_bandwidth(
        &mut self,
        clocks: &clocks::Clocks,
        rx_bw_hz: u32,
        tx_bw_hz: u32,
    ) -> Result<(), InitError<Spi::Error>> {
        let real_rx_bw = rx_bw_hz / 2;
        let real_tx_bw = tx_bw_hz / 2;
        let bbpll_freq = clocks.bb_pll();

        let rx_bbf_div = self
            .calibrate_rx_bb_analog_filter(real_rx_bw, bbpll_freq)
            .await?;
        self.calibrate_tx_bb_analog_filter(real_tx_bw, bbpll_freq)
            .await?;
        self.calibrate_rx_tia(real_rx_bw).await?;
        self.calibrate_tx_bb_second_analog_filter(real_tx_bw)
            .await?;
        self.setup_rx_adc(bbpll_freq, clocks.adc(), rx_bbf_div)
            .await?;
        Ok(())
    }

    async fn setup_trx_clock_chain(
        &mut self,
        reference_clk_rate: u32,
        clock: &clocks::ClockConfig,
        elna: Option<&config::ExternalLnaConfig>,
        gain_control: &config::GainControl,
        aux_adc: &config::AuxAdcConfig,
    ) -> Result<Clocks, InitError<Spi::Error>> {
        self.configure_bb_pll_clock(&clock.bb_pll).await?;
        self.configure_adc_clock(clock.adc).await?;
        self.configure_dac_clock(clock.tx.dac_div2).await?;
        self.configure_rx_hb_clock_chain(&clock.rx).await?;
        self.configure_tx_clock_chain(&clock.tx).await?;
        let clocks = Clocks::new(reference_clk_rate, clock);

        // The C driver also runs its digital tune here. That needs the AXI ADC and DAC, so the
        // caller has to run [`Ad9361::digital_tune`] afterwards.

        // The C driver calls this as some sort of post BB PLL change method. In the context of
        // the setup, this means doing this stuff twice. I do not know whether this is important,
        // but let's stick close to the C driver for now.
        //
        // TODO: RSSI and AUXADC seem really unnecessary. Even the gain control update might be.
        // TODO: Test whether the AUXADC update can be removed to avoid doing it twice.
        self.bb_pll_changed(&clocks, elna, gain_control, aux_adc)
            .await?;

        Ok(clocks)
    }

    async fn bb_pll_changed(
        &mut self,
        clocks: &Clocks,
        //rssi_config: &RssiConfig,
        elna_config: Option<&ExternalLnaConfig>,
        gain_config: &GainControl,
        aux_adc_config: &AuxAdcConfig,
    ) -> Result<(), Spi::Error> {
        self.update_gain_control(clocks, elna_config, gain_config)
            .await?;
        //self.setup_rssi(clocks, rssi_config, false).await?;
        self.setup_auxadc(clocks, aux_adc_config).await?;
        Ok(())
    }

    async fn update_gain_control(
        &mut self,
        clocks: &Clocks,
        elna_config: Option<&ExternalLnaConfig>,
        gain_config: &GainControl,
    ) -> Result<(), Spi::Error> {
        let rx_fir_input = clocks.rx().rx_fir_input();
        let delay_lna = elna_config.map_or(0, |v| v.settling_delay_ns);

        if let Some(agc) = gain_config.auto() {
            // AGC Attack Delay (us)=ceiling((((0.2+Delay_LNA)*ClkRF+14))/(2*ClkRF))+1
            // ClkRF in MHz, delay in us
            let reg = (200 + delay_lna) / 2 + (14_000_000 / (rx_fir_input / 500));
            // Convert ns to us (ceiling), then add extra margin
            let reg = reg.div_ceil(1000) + agc.attack_delay_extra_margin_us;
            let delay_val = reg.clamp(0, 31) as u8;
            self.modify_register(regs::Register::AgcAttackDelay, |val| {
                let mut reg = regs::agc_attack_delay::Register::new_with_raw_value(val);
                reg.set_agc_attack_delay_us(u6::new(delay_val));
                reg.raw_value()
            })
            .await?;
        }

        // Peak Overload Wait Time (ClkRF cycles)=ceiling((0.1+Delay_LNA) *clkRF+1)
        // delay_lna in ns, 100 = 0.1us in ns, clkrf in Hz
        let reg = (delay_lna + 100) * (rx_fir_input / 1000);
        let reg = reg.div_ceil(1_000_000) + 1;
        let raw_val = reg.clamp(0, 31) as u8;
        self.modify_register(regs::Register::PeakWaitTime, |val| {
            let mut reg = regs::peak_wait_time::Register::new_with_raw_value(val);
            reg.set_peak_overload_wait_time(u5::new(raw_val));
            reg.raw_value()
        })
        .await?;

        // Settling Delay in 0x111. Applies to all gain control modes:
        // 0x111[D4:D0]= ceiling(((0.2+Delay_LNA)*clkRF))
        // delay_lna in ns, 200 = 0.2us in ns, clkrf in Hz
        let reg = (delay_lna + 200) * (rx_fir_input / 2000);
        let reg = reg.div_ceil(1_000_000) + 7;
        let settling_delay = reg.clamp(0, 31) as u8;
        self.modify_register(regs::Register::PeakWaitTime, |mut val| {
            val &= !0b11111;
            val |= settling_delay;
            val
        })
        .await?;

        // Gain Update Counter [15:0]= round((((time*ClkRF-0x111[D4:D0]*2)-2))/2)
        // gain_update_interval_us in us, clkrf in Hz, settling_delay in cycles
        //
        // The C driver wraps around if the interval is too short, which ends up as the maximum
        // counter. We saturate to zero instead.
        let reg = (gain_config.common.gain_update_interval_us * (rx_fir_input / 1000))
            .saturating_sub(settling_delay as u32 * 2000 + 2000);
        let gain_update_counter = div_round_u32(reg, 2000).clamp(0, 131071u32);

        let dur_value = if gain_config.fast().is_some() {
            gain_config.common.dec_pow_measurement_duration.0
        } else {
            let fir_div = div_round_u32(rx_fir_input, clocks.rx().rx_sample());
            if ((gain_update_counter * 2 / fir_div)
                / gain_config.common.dec_pow_measurement_duration.rx_cycles())
                < 2
            {
                let dur_rx_cycles = gain_update_counter / fir_div;
                u4::new((dur_rx_cycles / 16).max(1).ilog2().min(u4::MAX.as_u32()) as u8)
            } else {
                gain_config.common.dec_pow_measurement_duration.0
            }
        };

        self.modify_register(regs::Register::DecPowerMeasureDuration, |val| {
            regs::dec_power_measure_duration::Register::new_with_raw_value(val)
                .with_dec_power_measurement_duration(dur_value)
                .raw_value()
        })
        .await?;

        self.modify_register(regs::Register::DigitalSaturationCounter, |val| {
            let mut reg = regs::digital_sat_counter::Register::new_with_raw_value(val);
            reg.set_double_gain_counter(gain_update_counter > 65535);
            reg.raw_value()
        })
        .await?;

        let gain_update_counter_reg = if gain_update_counter > 65535 {
            gain_update_counter / 2
        } else {
            gain_update_counter
        };

        self.write_register(
            regs::Register::GainUpdateCounter1,
            (gain_update_counter_reg & 0xff) as u8,
        )
        .await?;
        self.write_register(
            regs::Register::GainUpdateCounter2,
            (gain_update_counter_reg >> 8) as u8,
        )
        .await?;

        if let Some(fast) = gain_config.fast() {
            let reg = div_round_u32(fast.state_wait_time_ns * (rx_fir_input / 1000), 1_000_000);
            let reg = reg.clamp(0, 31) as u8;
            self.modify_register(regs::Register::FastEnergyDetectCount, |mut val| {
                val &= !0b11111;
                val |= reg;
                val
            })
            .await?;
        }
        Ok(())
    }

    async fn configure_adc_clock(&mut self, adc_div: AdcDivisor) -> Result<(), Spi::Error> {
        self.modify_register(regs::Register::BbPll, |val| {
            let mut reg = regs::clk_bb_pll::Register::new_with_raw_value(val);
            reg.set_bb_pll_div(adc_div);
            reg.raw_value()
        })
        .await?;
        Ok(())
    }

    async fn configure_dac_clock(&mut self, dac_div2: bool) -> Result<(), Spi::Error> {
        self.modify_register(regs::Register::BbPll, |val| {
            let mut reg = regs::clk_bb_pll::Register::new_with_raw_value(val);
            reg.set_dac_clk_div2(dac_div2);
            reg.raw_value()
        })
        .await?;
        Ok(())
    }

    /// Sets up the fixed RX half-band decimation stages (RHB1-3) at their real target values, but
    /// leaves the general-purpose RX FIR bypassed regardless of `config.rx_fir`. The AD9361's FIR
    /// coefficient RAM hasn't been loaded with real taps yet at this point in `init()`. Enabling
    /// it here would mean calibration, which runs shortly after, sees whatever garbage or
    /// POR-default coefficients happen to be sitting in RAM. [`Ad9361Uninit::init`] re-enables it
    /// at the real target value once calibration is done, mirroring `phy->bypass_rx_fir` in the C
    /// driver (`ad9361_clear_state`/`ad9361_set_trx_clock_chain` in ad9361.c).
    async fn configure_rx_hb_clock_chain(
        &mut self,
        config: &clocks::RxConfig,
    ) -> Result<(), Spi::Error> {
        self.modify_register(regs::Register::RxEnableFilterCtrl, |val| {
            let mut reg = regs::rx_enable_filter_ctrl::Register::new_with_raw_value(val);
            reg.set_rhb3(config.rhb3);
            reg.set_rhb2(config.rhb2);
            reg.set_rhb1(config.rhb1);
            reg.set_rx_fir(clocks::RxFirDecimation::Div1BypassFilter);
            reg.raw_value()
        })
        .await?;
        Ok(())
    }

    /// Sets up the fixed TX half-band interpolation stages (THB1-3) at their real target values,
    /// but leaves the general-purpose TX FIR bypassed regardless of `config.tx_fir`. See
    /// [`Self::configure_rx_hb_clock_chain`] for why.
    async fn configure_tx_clock_chain(
        &mut self,
        config: &clocks::TxConfig,
    ) -> Result<(), Spi::Error> {
        self.modify_register(regs::Register::TxEnableFilterCtrl, |val| {
            let mut reg = regs::tx_enable_filter_ctrl::Register::new_with_raw_value(val);
            reg.set_thb3(config.thb3);
            reg.set_thb2(config.thb2);
            reg.set_thb1(config.thb1);
            reg.set_tx_fir(clocks::TxFirInterpolation::Mult1BypassFilter);
            reg.raw_value()
        })
        .await?;
        Ok(())
    }

    /// Enables the RX/TX FIR stages at their configured target ratios (`clock.rx.rx_fir`/
    /// `clock.tx.tx_fir`). Called at the end of [`Ad9361Uninit::init`], after calibration.
    /// Calibration needs the FIR bypassed, see [`Self::configure_rx_hb_clock_chain`] for why.
    ///
    /// This only flips the enable/ratio bits. It does not load any filter taps. If either target
    /// ratio isn't a bypass variant, the caller is responsible for loading real coefficients
    /// afterward via [`Ad9361::set_tx_fir_config`] and [`Ad9361::set_rx_fir_config`]. Until
    /// then, the FIR runs with whatever's already in its coefficient RAM, which is typically all
    /// zero after a fresh power-on reset.
    async fn enable_configured_fir(
        &mut self,
        clock: &clocks::ClockConfig,
    ) -> Result<(), Spi::Error> {
        self.modify_register(regs::Register::RxEnableFilterCtrl, |val| {
            regs::rx_enable_filter_ctrl::Register::new_with_raw_value(val)
                .with_rx_fir(clock.rx.rx_fir)
                .raw_value()
        })
        .await?;
        self.modify_register(regs::Register::TxEnableFilterCtrl, |val| {
            regs::tx_enable_filter_ctrl::Register::new_with_raw_value(val)
                .with_tx_fir(clock.tx.tx_fir)
                .raw_value()
        })
        .await?;
        Ok(())
    }

    #[allow(clippy::await_holding_refcell_ref)]
    async fn configure_bb_pll_clock(
        &mut self,
        config: &clocks::BbPllConfig,
    ) -> Result<(), InitError<Spi::Error>> {
        const LF_DEFAULTS: [u8; 3] = [0x35, 0x5B, 0xE8];
        self.write_register(
            regs::Register::CpCurrent,
            config.charge_pump.reg_value().value(),
        )
        .await?;
        let mut buf: [u8; 5] = [0; 5];
        // AD9361 decrements on multi-writes.
        spi::prepare_spi_write(&mut buf, regs::Register::LoopFilter3.as_u10(), u3::new(3));
        buf[2..5].copy_from_slice(&LF_DEFAULTS);
        self.spi.borrow_mut().write(&buf).await?;

        // Allow calibration to occur and set cal count to 1024 for max accuracy
        self.write_register(
            regs::Register::VcoCtrl,
            regs::vco_ctrl::Register::ZERO
                .with_freq_cal_enable(true)
                .with_should_be_one(u2::new(0b11))
                .raw_value(),
        )
        .await?;
        // Set calibration clock to REFCLK/4 for more accuracy
        self.write_register(regs::Register::SdmCtrl, 0x10).await?;

        self.write_register(regs::Register::IntegerBbFreqWord, config.integer_freq_word)
            .await?;
        self.write_register(
            regs::Register::FractBbFreqWord3,
            (config.fractional_freq_word.value() & 0xff) as u8,
        )
        .await?;
        self.write_register(
            regs::Register::FractBbFreqWord2,
            ((config.fractional_freq_word.value() >> 8) & 0xff) as u8,
        )
        .await?;
        self.write_register(
            regs::Register::FractBbFreqWord1,
            ((config.fractional_freq_word.value() >> 16) & 0xff) as u8,
        )
        .await?;

        // Start calibration.
        self.write_register(
            regs::Register::BbPllCtrl1,
            regs::sdm_ctrl_1::INIT_BB_FO_CAL | regs::sdm_ctrl_1::BBPLL_RESET_BAR,
        )
        .await?;
        // Clear calibration bit, not self-clearing.
        self.write_register(
            regs::Register::BbPllCtrl1,
            regs::sdm_ctrl_1::BBPLL_RESET_BAR,
        )
        .await?;

        // Increase BBPLL KV and phase margin.
        self.write_register(regs::Register::VcoProgram1, 0x86)
            .await?;
        self.write_register(regs::Register::VcoProgram2, 0x01)
            .await?;
        self.write_register(regs::Register::VcoProgram2, 0x05)
            .await?;

        let mut elapsed = 0;
        loop {
            let reg = self.read_register(regs::Register::Ch1Overflow).await?;
            if (reg >> 7) & 0b1 == 1 {
                break;
            }
            if elapsed > 20_000 {
                return Err(InitError::CalibrationTimeout(ModuleId::BbPll));
            }
            self.delay.delay_us(120).await;
            elapsed += 120;
        }
        Ok(())
    }

    async fn setup_auxadc(
        &mut self,
        clocks: &Clocks,
        config: &config::AuxAdcConfig,
    ) -> Result<(), Spi::Error> {
        let val = div_round_u64(
            config.temp_sense.measurement_interval_ms as u64 * (clocks.bb_pll() as u64 / 1000),
            1 << 29,
        ) as u32;
        self.write_register(
            regs::Register::TempOffset,
            config.temp_sense.offset_signed as u8,
        )
        .await?;
        self.write_register(regs::Register::StartTempReading, 0)
            .await?;
        self.write_register(
            regs::Register::TempSense2,
            regs::temp_sense2::Register::builder()
                .with_measurement_time_interval(u7::new(val.clamp(0, u7::MAX.as_u32()) as u8))
                .with_temp_sense_periodic_enable(config.temp_sense.enable_periodic)
                .build()
                .raw_value(),
        )
        .await?;
        self.write_register(
            regs::Register::TempSensorConfig,
            regs::temp_sensor_config::Register::builder()
                .with_decimation(config.temp_sense.decimation.0)
                .build()
                .raw_value(),
        )
        .await?;

        self.write_register(
            regs::Register::AuxadcClockDivider,
            clocks
                .bb_pll()
                .div_ceil(config.clock_rate_hz.into())
                .saturating_sub(1)
                .min(u6::MAX.as_u32()) as u8,
        )
        .await?;
        self.write_register(
            regs::Register::AuxadcConfig,
            regs::auxadc_config::Register::builder()
                .with_decimation(config.decimation.0)
                .with_power_down(false)
                .build()
                .raw_value(),
        )
        .await?;
        Ok(())
    }

    async fn setup_rssi(
        &mut self,
        clocks: &Clocks,
        rssi_config: &RssiConfig,
        update: bool,
    ) -> Result<(), Spi::Error> {
        let (mut rssi_delay, rssi_duration, mut rssi_wait) = match rssi_config.inner {
            config::TemporalConfig::RxSamples {
                delay,
                duration,
                wait,
            } => {
                if update {
                    return Ok(());
                }
                (delay, duration, wait)
            }
            config::TemporalConfig::TimeUs {
                delay_us,
                duration_us,
                wait_us,
            } => {
                // update sample based on RX rate, units are in us
                let rate = div_round_u32(clocks.rx().rx_sample(), 1000);
                (
                    div_round_u32(delay_us * rate, 1000),
                    div_round_u32(duration_us * rate, 1000),
                    div_round_u32(wait_us * rate, 1000),
                )
            }
        };
        if rssi_config.restart_mode == config::RssiRestartMode::EnAgcPinIsPulledHigh {
            rssi_delay = 0;
        }
        rssi_delay = (rssi_delay / 8).clamp(0, 255);
        rssi_wait = (rssi_wait / 4).clamp(0, 255);

        /*
         * C variant (direct translation, kept as reference):
         *
         * let mut dur_buf = [0u8; 4];
         * let mut j = 0usize;
         * let mut total_dur = 0i64;
         * let mut rssi_duration = rssi_duration as i64;
         *
         * loop {
         *     let mut found = false;
         *     for i in (0u8..=14).rev() {
         *         let val = 1i64 << i;
         *         if rssi_duration >= val {
         *             dur_buf[j] = i;
         *             j += 1;
         *             total_dur += val;
         *             rssi_duration -= val;
         *             found = true;
         *             break;
         *         }
         *     }
         *     if !found || j >= 4 || rssi_duration <= 0 {
         *         break;
         *     }
         * }
         *
         * let mut weight = [0i32; 4];
         * let mut total_weight = 0u32;
         * for i in 0..4 {
         *     weight[i] = if i < j {
         *         div_round(
         *             RSSI_MAX_WEIGHT as u32 * (1 << dur_buf[i]) as u32,
         *             total_dur as u32,
         *         ) as i32
         *     } else {
         *         0
         *     };
         *     total_weight += weight[i] as u32;
         * }
         * if j > 0 {
         *     // total of all weights must be 0xFF
         *     weight[j - 1] -= total_weight as i32 - 0xff;
         * }
         */
        // Greedily decompose rssi_duration into up to 4 power-of-two terms (largest first).
        let mut rssi_duration = rssi_duration as i64;
        let mut dur_buf: heapless::Vec<u8, 4> = (0u8..=14)
            .rev()
            .filter(|&i| {
                let val = 1i64 << i;
                if rssi_duration >= val {
                    rssi_duration -= val;
                    true
                } else {
                    false
                }
            })
            .take(4)
            .collect();

        let total_dur: u32 = dur_buf.iter().map(|&i| 1u32 << i).sum::<u32>().max(1);

        let mut weight: heapless::Vec<u32, 4> = dur_buf
            .iter()
            .map(|&i| div_round_u32(RSSI_MAX_WEIGHT as u32 * (1u32 << i), total_dur))
            .collect();

        // Adjust last weight so all weights sum to exactly 0xFF
        let total_weight: u32 = weight.iter().sum();
        let correction = total_weight.saturating_sub(0xFF);
        if let Some(last) = weight.last_mut() {
            *last -= correction;
        }

        // Pad to 4 entries for register writes
        let n_terms = dur_buf.len();
        while dur_buf.len() < 4 {
            let _ = dur_buf.push(0);
        }
        while weight.len() < 4 {
            let _ = weight.push(0);
        }

        self.write_register(
            regs::Register::MeasureDuration01,
            regs::measure_duration_01::Register::builder()
                .with_duration_1(u4::new(dur_buf[1] & 0xF))
                .with_duration_0(u4::new(dur_buf[0] & 0xF))
                .build()
                .raw_value(),
        )
        .await?;
        self.write_register(
            regs::Register::MeasureDuration23,
            regs::measure_duration_23::Register::builder()
                .with_duration_3(u4::new(dur_buf[3] & 0xF))
                .with_duration_2(u4::new(dur_buf[2] & 0xF))
                .build()
                .raw_value(),
        )
        .await?;
        self.write_register(regs::Register::RssiWeight0, weight[0] as u8)
            .await?;
        self.write_register(regs::Register::RssiWeight1, weight[1] as u8)
            .await?;
        self.write_register(regs::Register::RssiWeight2, weight[2] as u8)
            .await?;
        self.write_register(regs::Register::RssiWeight3, weight[3] as u8)
            .await?;
        self.write_register(regs::Register::RssiDelay, rssi_delay as u8)
            .await?;
        self.write_register(regs::Register::RssiWaitTime, rssi_wait as u8)
            .await?;

        let rssi_cfg = regs::rssi_config::Register::builder()
            .with_rfir_for_rssi_measurement(u2::new(0))
            .with_restart_mode(rssi_config.restart_mode)
            .with_start_rssi_meas(
                rssi_config.restart_mode == config::RssiRestartMode::SpiWriteToRegister,
            )
            .with_should_be_zero(false)
            .with_default_rssi_meas_mode(rssi_duration == 0 && n_terms == 1)
            .build();
        self.write_register(regs::Register::RssiConfig, rssi_cfg.raw_value())
            .await?;

        Ok(())
    }
}

/// The set of registers that implement the coefficient-RAM write protocol used by both the TX
/// and RX FIR filters (address/data/config/dummy-read-delay), looked up once per direction so
/// [`Ad9361::load_fir_coefficients`] can run the identical sequence against either one.
struct FirRegisterSet {
    enable_ctrl: regs::Register,
    coef_addr: regs::Register,
    coef_data1: regs::Register,
    coef_data2: regs::Register,
    coef_read_delay: regs::Register,
    filter_conf: regs::Register,
}

impl FirRegisterSet {
    const TX: Self = Self {
        enable_ctrl: regs::Register::TxEnableFilterCtrl,
        coef_addr: regs::Register::TxFilterCoefAddr,
        coef_data1: regs::Register::TxFilterCoefWriteData1,
        coef_data2: regs::Register::TxFilterCoefWriteData2,
        coef_read_delay: regs::Register::TxFilterCoefReadData2,
        filter_conf: regs::Register::TxFilterConf,
    };
    const RX: Self = Self {
        enable_ctrl: regs::Register::RxEnableFilterCtrl,
        coef_addr: regs::Register::RxFilterCoefAddr,
        coef_data1: regs::Register::RxFilterCoefData1,
        coef_data2: regs::Register::RxFilterCoefData2,
        coef_read_delay: regs::Register::RxFilterCoefReadData2,
        filter_conf: regs::Register::RxFilterConfig,
    };

    /// Start-clock bit, bit 1 of both `TxFilterConf` and `RxFilterConfig`
    /// (`tx_filter_conf::Register::start_tx_clock` / `rx_filter_config::Register::start_rx_clock`).
    const FILTER_CONF_START_CLOCK: u8 = 1 << 1;
    /// Write-strobe bit, bit 2 of both `TxFilterConf` and `RxFilterConfig`
    /// (`tx_filter_conf::Register::write_tx` / `rx_filter_config::Register::write_rx`).
    const FILTER_CONF_WRITE: u8 = 1 << 2;
}

/// Error from [`Ad9361::read_temperature_millicelsius`].
#[derive(Debug, thiserror::Error)]
pub enum TemperatureReadError<Spi> {
    /// SPI error.
    #[error("SPI error: {0}")]
    Spi(#[from] Spi),
    /// Manual mode only: the temperature sensor valid signal (Register 0x00C, Bit D1) did not
    /// toggle within the timeout.
    #[error("temperature sensor valid signal did not toggle within the timeout")]
    Timeout,
}

#[bisync]
impl<Spi: SpiDevice, ResetPin: OutputPin, Delay: DelayNs> Ad9361<Spi, ResetPin, Delay> {
    /// Reads the product ID and silicon revision registers.
    pub async fn read_product_id(&mut self) -> Result<ProductIdReg, Spi::Error> {
        self.inner.read_product_id().await
    }

    /// Writes a single register.
    pub async fn write_register(&mut self, reg: regs::Register, val: u8) -> Result<(), Spi::Error> {
        self.inner.write_register(reg, val).await
    }

    /// Writes a single register by its raw 10-bit SPI address, bypassing the [`regs::Register`]
    /// enum.
    pub async fn write_register_raw(&mut self, reg: u10, val: u8) -> Result<(), Spi::Error> {
        self.inner.write_register_raw(reg, val).await
    }

    /// Reads a single register by its raw 10-bit SPI address, bypassing the [`regs::Register`]
    /// enum.
    pub async fn read_register_raw(&self, reg: u10) -> Result<u8, Spi::Error> {
        self.inner.read_register_raw(reg).await
    }

    /// Reads a single register.
    pub async fn read_register(&self, reg: regs::Register) -> Result<u8, Spi::Error> {
        self.inner.read_register(reg).await
    }

    /// Reads the AD9361 die temperature, in millidegrees Celsius.
    ///
    /// Adapts to whichever mode [`crate::config::TempSenseConfig::enable_periodic`] is currently
    /// configured with, by checking Register 0x00D Bit D0 directly rather than assuming a mode:
    ///
    /// - Periodic mode: the chip keeps the temperature register refreshed on its own schedule, so
    ///   this is a plain read, matching the ADI reference driver's `ad9361_get_temp`.
    /// - Manual mode: per UG-570's Register 0x00C bit reference, Bit D0 ("Start Temp Reading") is
    ///   edge-triggered and not self-clearing, so a new reading requires clearing it and setting
    ///   it again even if it was already set. This then polls Bit D1, which toggles once the new
    ///   reading is valid, until it differs from its pre-trigger state, with a timeout.
    ///
    /// Either way, the AuxADC is powered down while reading the raw temperature register and
    /// restored afterward (UG-570's register reference for 0x00E: "disable the AuxADC ... to
    /// ensure a valid temperature reading"), and the raw code is converted with the same
    /// `raw * 1_000_000 / 1140` scaling ADI's reference driver uses (rounded to the nearest
    /// integer).
    pub async fn read_temperature_millicelsius(
        &mut self,
    ) -> Result<i32, TemperatureReadError<Spi::Error>> {
        const POLL_INTERVAL_US: u32 = 200;
        const POLL_TIMEOUT_US: u32 = 50_000;

        let periodic_enabled = regs::temp_sense2::Register::new_with_raw_value(
            self.read_register(regs::Register::TempSense2).await?,
        )
        .temp_sense_periodic_enable();

        if !periodic_enabled {
            let mut start_temp_reading = regs::start_temp_reading::Register::new_with_raw_value(
                self.read_register(regs::Register::StartTempReading).await?,
            );
            let valid_before = start_temp_reading.temp_sensor_valid();
            start_temp_reading.set_start_temp_reading(false);
            self.write_register(
                regs::Register::StartTempReading,
                start_temp_reading.raw_value(),
            )
            .await?;
            start_temp_reading.set_start_temp_reading(true);
            self.write_register(
                regs::Register::StartTempReading,
                start_temp_reading.raw_value(),
            )
            .await?;

            let mut elapsed_us = 0;
            loop {
                let current = regs::start_temp_reading::Register::new_with_raw_value(
                    self.read_register(regs::Register::StartTempReading).await?,
                );
                if current.temp_sensor_valid() != valid_before {
                    break;
                }
                if elapsed_us >= POLL_TIMEOUT_US {
                    return Err(TemperatureReadError::Timeout);
                }
                self.inner.delay.delay_us(POLL_INTERVAL_US).await;
                elapsed_us += POLL_INTERVAL_US;
            }
        }

        let mut auxadc_config = regs::auxadc_config::Register::new_with_raw_value(
            self.read_register(regs::Register::AuxadcConfig).await?,
        );
        auxadc_config.set_power_down(true);
        self.write_register(regs::Register::AuxadcConfig, auxadc_config.raw_value())
            .await?;

        let raw = self.read_register(regs::Register::Temperature).await?;

        auxadc_config.set_power_down(false);
        self.write_register(regs::Register::AuxadcConfig, auxadc_config.raw_value())
            .await?;

        Ok(((i64::from(raw) * 1_000_000 + 570) / 1140) as i32)
    }

    /// Configures which internal status signal each of the 4 control output (`CTRL_OUT`) pins
    /// reflects. `pointer` selects the signal group, `enable_bits` is a mask selecting which of
    /// the 4 pins within that group are driven.
    pub async fn configure_control_output_pins(
        &mut self,
        pointer: u5,
        enable_bits: u8,
    ) -> Result<(), Spi::Error> {
        self.write_register(regs::Register::CtrlOutputPointer, pointer.value())
            .await?;
        self.write_register(regs::Register::CtrlOutputEnable, enable_bits)
            .await?;
        Ok(())
    }

    /// Mutes both TX channels (sets attenuation to the maximum, 89750 mdB) and returns a guard
    /// holding the previous per-channel attenuation.
    ///
    /// `Drop` cannot perform the async SPI write needed to restore the previous attenuation, so
    /// the guard must be explicitly un-muted via [`TxMuteGuard::unmute`] — see that method's docs.
    pub async fn mute_tx(&mut self) -> Result<TxMuteGuard, Spi::Error> {
        let (tx1_md_b, tx2_md_b) = self.tx_mute().await?;
        Ok(TxMuteGuard {
            tx1_md_b,
            tx2_md_b,
            restored: false,
        })
    }

    async fn tx_mute(&mut self) -> Result<(u32, u32), Spi::Error> {
        let tx1_md_b = self.get_tx_atten(TransmitterId::Tx1).await?;
        let tx2_md_b = self.get_tx_atten(TransmitterId::Tx2).await?;
        self.set_tx_atten_raw(89750, true, true).await?;
        Ok((tx1_md_b, tx2_md_b))
    }

    async fn tx_unmute(&mut self, tx1_md_b: u32, tx2_md_b: u32) -> Result<(), Spi::Error> {
        if tx1_md_b == tx2_md_b {
            self.set_tx_atten_raw(tx1_md_b, true, true).await
        } else {
            self.set_tx_atten_raw(tx1_md_b, true, false).await?;
            self.set_tx_atten_raw(tx2_md_b, false, true).await
        }
    }

    async fn get_tx_atten(&mut self, tx: TransmitterId) -> Result<u32, Spi::Error> {
        let (msb_reg, lsb_reg) = match tx {
            TransmitterId::Tx1 => (regs::Register::Tx1Atten1, regs::Register::Tx1Atten0),
            TransmitterId::Tx2 => (regs::Register::Tx2Atten1, regs::Register::Tx2Atten0),
        };
        let msb = self.read_register(msb_reg).await?;
        let lsb = self.read_register(lsb_reg).await?;
        Ok((((msb as u32) << 8) | lsb as u32) * 250)
    }

    async fn set_tx_atten_raw(
        &mut self,
        atten_md_b: u32,
        tx1: bool,
        tx2: bool,
    ) -> Result<(), Spi::Error> {
        let code = (atten_md_b / 250) as u16;

        let dig_atten = self.read_register(regs::Register::Tx2DigAtten).await?;
        self.write_register(
            regs::Register::Tx2DigAtten,
            regs::tx2_dig_atten::Register::new_with_raw_value(dig_atten)
                .with_immediately_update_tpc_atten(false)
                .raw_value(),
        )
        .await?;

        if tx1 {
            self.write_register(regs::Register::Tx1Atten1, (code >> 8) as u8)
                .await?;
            self.write_register(regs::Register::Tx1Atten0, (code & 0xff) as u8)
                .await?;
        }
        if tx2 {
            self.write_register(regs::Register::Tx2Atten1, (code >> 8) as u8)
                .await?;
            self.write_register(regs::Register::Tx2Atten0, (code & 0xff) as u8)
                .await?;
        }

        let dig_atten = self.read_register(regs::Register::Tx2DigAtten).await?;
        self.write_register(
            regs::Register::Tx2DigAtten,
            regs::tx2_dig_atten::Register::new_with_raw_value(dig_atten)
                .with_immediately_update_tpc_atten(true)
                .raw_value(),
        )
        .await?;

        Ok(())
    }

    /// Shared coefficient-loading core of [`set_tx_fir_config`](Self::set_tx_fir_config) and
    /// [`set_rx_fir_config`](Self::set_rx_fir_config), mirroring the common part of
    /// `ad9361_load_fir_filter_coef` in the C driver: force the enable/ratio register to
    /// `enable_ctrl_target`, start the clock, walk the coefficient table with the write-strobe +
    /// dummy-read delay pattern, then restore `enable_ctrl_before`.
    ///
    /// `filter_conf_base` must already have every bit set except the write-strobe and
    /// start-clock ones (number of taps, channel select, and TX's 6dB attenuate bit) — this
    /// only ever touches [`FirRegisterSet::FILTER_CONF_WRITE`] and
    /// [`FirRegisterSet::FILTER_CONF_START_CLOCK`], which sit at the same bit position in both
    /// `TxFilterConf` and `RxFilterConfig`.
    async fn load_fir_coefficients(
        &mut self,
        regs_set: FirRegisterSet,
        enable_ctrl_before: u8,
        enable_ctrl_target: u8,
        filter_conf_base: u8,
        num_taps: u8,
        coefs: &[i16],
    ) -> Result<(), Spi::Error> {
        self.inner
            .write_register(regs_set.enable_ctrl, enable_ctrl_target)
            .await?;

        let filter_conf = filter_conf_base | FirRegisterSet::FILTER_CONF_START_CLOCK;
        self.write_register(regs_set.filter_conf, filter_conf)
            .await?;

        for (i, coef) in coefs.iter().take(num_taps as usize).enumerate() {
            self.write_register(regs_set.coef_addr, i as u8).await?;
            self.write_register(regs_set.coef_data1, (coef & 0xff) as u8)
                .await?;
            self.write_register(regs_set.coef_data2, ((coef >> 8) & 0xff) as u8)
                .await?;
            self.write_register(
                regs_set.filter_conf,
                filter_conf | FirRegisterSet::FILTER_CONF_WRITE,
            )
            .await?;
            // Dummy writes for delay.
            self.write_register(regs_set.coef_read_delay, 0).await?;
            self.write_register(regs_set.coef_read_delay, 0).await?;
        }
        self.write_register(regs_set.filter_conf, filter_conf)
            .await?;
        self.write_register(
            regs_set.filter_conf,
            filter_conf & !FirRegisterSet::FILTER_CONF_START_CLOCK,
        )
        .await?;

        self.inner
            .write_register(regs_set.enable_ctrl, enable_ctrl_before)
            .await?;
        Ok(())
    }

    /// Loads the TX FIR taps for the TX ratio of the active clock configuration. The taps depend
    /// on that ratio, so reload them after [`Self::update_rf_clocks`] changed it.
    ///
    /// Fails without touching the hardware if there are more taps than the clock ratio allows.
    pub async fn set_tx_fir_config<'coef>(
        &mut self,
        config: &config::TxFirConfig<'coef>,
    ) -> Result<(), InitError<Spi::Error>> {
        let num_taps = config.num_taps.number();
        self.config
            .check_fir_taps(&self.config.clock, Some(num_taps), None)?;
        let tx_fir = self.config.clock.tx.tx_fir;
        let previous_ensm_state = self.force_ensm_state(regs::EnsmState::Alert).await?;

        // Mirrors `ad9361_load_fir_filter_coef` in the C driver: read back whatever is
        // currently programmed so it can be restored afterward, then unconditionally switch
        // the interpolation ratio to the active one for the duration of the coefficient load
        // (the C driver always writes the active ratio here, it does not only touch the
        // register in some special-cased "already at max ratio" branch).
        let filter_control = regs::tx_enable_filter_ctrl::Register::new_with_raw_value(
            self.read_register(regs::Register::TxEnableFilterCtrl)
                .await?,
        );
        let filter_conf_base = regs::tx_filter_conf::Register::ZERO
            .with_write_goes_to_tx2(config.ch_select.channel2())
            .with_write_goes_to_tx1(config.ch_select.channel1())
            .with_number_of_taps(config.num_taps)
            .with_attentuate_6db(config.attentuate_6db)
            .raw_value();

        self.load_fir_coefficients(
            FirRegisterSet::TX,
            filter_control.raw_value(),
            filter_control.with_tx_fir(tx_fir).raw_value(),
            filter_conf_base,
            num_taps,
            config.tx_coefs,
        )
        .await?;
        self.config.tx_fir_taps = Some(num_taps);

        // Ignore errors here, we are basically done.
        let _ = self.restore_ensm_state(previous_ensm_state).await;
        Ok(())
    }

    /// Loads the RX FIR taps for the RX ratio of the active clock configuration. The taps depend
    /// on that ratio, so reload them after [`Self::update_rf_clocks`] changed it.
    ///
    /// Fails without touching the hardware if there are more taps than the clock ratio allows.
    pub async fn set_rx_fir_config<'coef>(
        &mut self,
        config: &config::RxFirConfig<'coef>,
    ) -> Result<(), InitError<Spi::Error>> {
        let num_taps = config.num_taps.number();
        self.config
            .check_fir_taps(&self.config.clock, None, Some(num_taps))?;
        let rx_fir = self.config.clock.rx.rx_fir;
        let previous_ensm_state = self.force_ensm_state(regs::EnsmState::Alert).await?;

        // Unlike TX (`attentuate_6db` lives in `TxFilterConf` itself and is toggled per load),
        // RX's filter gain is a separate register that the C driver writes once, up front, and
        // never saves/restores.
        self.write_register(
            regs::Register::RxFilterGain,
            regs::rx_filter_gain::Register::ZERO
                .with_filter_gain(config.gain)
                .raw_value(),
        )
        .await?;

        // Mirrors `ad9361_load_fir_filter_coef` in the C driver: read back whatever is
        // currently programmed so it can be restored afterward, then unconditionally switch
        // the decimation ratio to the active one for the duration of the coefficient load
        // (the C driver always writes the active ratio here, it does not only touch the
        // register in some special-cased "already at max ratio" branch).
        let filter_control = regs::rx_enable_filter_ctrl::Register::new_with_raw_value(
            self.read_register(regs::Register::RxEnableFilterCtrl)
                .await?,
        );
        let filter_conf_base = regs::rx_filter_config::Register::ZERO
            .with_write_goes_to_rx2(config.ch_select.channel2())
            .with_write_goes_to_rx1(config.ch_select.channel1())
            .with_number_of_taps(config.num_taps)
            .raw_value();

        self.load_fir_coefficients(
            FirRegisterSet::RX,
            filter_control.raw_value(),
            filter_control.with_rx_fir(rx_fir).raw_value(),
            filter_conf_base,
            num_taps,
            config.rx_coefs,
        )
        .await?;
        self.config.rx_fir_taps = Some(num_taps);

        // Ignore errors here, we are basically done.
        let _ = self.restore_ensm_state(previous_ensm_state).await;
        Ok(())
    }

    /// Applies a new clock path at runtime, for example to change the sample rate.
    ///
    /// Only the BB PLL, the ADC/DAC clocks and the half-band and FIR stages are reprogrammed.
    /// The reference clock scalers and the LO configuration stay as they were during setup,
    /// because the RF PLLs do not depend on the sample rate. The settings which depend on the
    /// new rates are updated as well: gain control, AUXADC, RSSI, baseband filters and the TX
    /// quad calibration.
    ///
    /// Fails without touching the hardware if the loaded FIR taps do not fit the new clock
    /// ratios.
    ///
    /// If the FIR ratios change, the caller must reload the filter taps afterwards with
    /// [`Self::set_tx_fir_config`] and [`Self::set_rx_fir_config`]. The digital interface timing
    /// also changes, so [`Self::digital_tune`] should be re-run.
    ///
    /// On error, the chip may be left in ALERT state with tracking disabled. The driver should be
    /// reset and set up again.
    ///
    /// Returns the new clocks.
    pub async fn update_rf_clocks(
        &mut self,
        path: &clocks::BbClockPathConfigHelper,
    ) -> Result<Clocks, InitError<Spi::Error>> {
        let clock = self.config.clock_for_path(path);
        self.config
            .check_fir_taps(&clock, self.config.tx_fir_taps, self.config.rx_fir_taps)?;

        // Unlike the C driver, ALERT state and disabled tracking also cover the clock chain
        // change, not only the calibrations.
        let previous_ensm_state = self.force_ensm_state(regs::EnsmState::Alert).await?;
        let saved_tracking = self.disable_tracking().await?;

        // This bypasses the FIR, it is enabled again at the end.
        let clocks = self
            .inner
            .setup_trx_clock_chain(
                self.config.reference_clk_rate,
                &clock,
                self.config.elna.as_ref(),
                &self.config.gain_control,
                &self.config.aux_adc,
            )
            .await?;
        self.inner
            .setup_rssi(&clocks, &self.config.rssi, true)
            .await?;
        self.inner
            .update_rf_bandwidth(
                &clocks,
                self.config.rf_rx_bandwidth_hz,
                self.config.rf_tx_bandwidth_hz,
            )
            .await?;
        if let Some(quad) = self.config.tx_quad {
            self.inner
                .perform_tx_quad_calibration(
                    self.config.transceiver_channel_mode,
                    &clocks,
                    self.config.rf_tx_bandwidth_hz,
                    self.config.rf_rx_bandwidth_hz,
                    quad.phase_config,
                    quad.gain_table_index,
                    quad.phase_inversion_en,
                )
                .await?;
        }
        self.inner.enable_configured_fir(&clock).await?;
        // The chip runs the new clocks from here on, so the stored config has to follow even if
        // one of the restore steps below fails.
        self.config.clock = clock;

        self.restore_tracking(saved_tracking).await?;
        self.restore_ensm_state(previous_ensm_state).await?;
        Ok(clocks)
    }

    /// Disables the BB DC, RF DC and quadrature tracking. The returned values are needed by
    /// [`Self::restore_tracking`].
    async fn disable_tracking(&mut self) -> Result<SavedTracking, Spi::Error> {
        let dc_offset_config_2 = self
            .inner
            .read_register(regs::Register::DcOffsetConfig2)
            .await?;
        let calibration_config_1 = self
            .inner
            .read_register(regs::Register::CalibrationConfig1)
            .await?;
        self.inner
            .write_register(
                regs::Register::DcOffsetConfig2,
                regs::dc_offset_config2::Register::new_with_raw_value(dc_offset_config_2)
                    .with_enable_bb_dc_offset_tracking(false)
                    .with_enable_rf_offset_tracking(false)
                    .raw_value(),
            )
            .await?;
        self.inner
            .write_register(
                regs::Register::CalibrationConfig1,
                regs::calibration_config_1::Register::new_with_raw_value(calibration_config_1)
                    .with_enable_tracking_mode_ch1(false)
                    .with_enable_tracking_mode_ch2(false)
                    .raw_value(),
            )
            .await?;
        Ok(SavedTracking {
            dc_offset_config_2,
            calibration_config_1,
        })
    }

    async fn restore_tracking(&mut self, saved: SavedTracking) -> Result<(), Spi::Error> {
        self.inner
            .write_register(regs::Register::DcOffsetConfig2, saved.dc_offset_config_2)
            .await?;
        self.inner
            .write_register(
                regs::Register::CalibrationConfig1,
                saved.calibration_config_1,
            )
            .await
    }

    async fn restore_ensm_state(
        &mut self,
        previous_state: regs::EnsmState,
    ) -> Result<(), EnsmError<Spi::Error>> {
        let mut config1 = regs::ensm_config_1::Register::new_with_raw_value(
            self.read_register(regs::Register::EnsmConfig1).await?,
        )
        .with_force_rx_on(false)
        .with_force_tx_on(false)
        .with_force_alert_state(false)
        .with_to_alert(true);
        match previous_state {
            regs::EnsmState::Alert => {
                config1.set_to_alert(true);
            }
            regs::EnsmState::Tx | regs::EnsmState::Fdd => {
                config1.set_force_tx_on(true);
            }
            regs::EnsmState::Rx => {
                config1.set_force_rx_on(true);
            }
            _ => {
                return Ok(());
            }
        }
        self.inner
            .write_register(
                regs::Register::EnsmConfig1,
                regs::ensm_config_1::Register::ZERO
                    .with_to_alert(true)
                    .with_force_alert_state(true)
                    .raw_value(),
            )
            .await?;
        self.inner
            .write_register(regs::Register::EnsmConfig1, config1.raw_value())
            .await?;
        if self.state.ensm_pin_control {
            config1.set_enable_ensm_pin_ctrl(true);
            self.inner
                .write_register(regs::Register::EnsmConfig1, config1.raw_value())
                .await?;
        }
        Ok(())
    }

    async fn force_ensm_state(
        &mut self,
        target_state: regs::EnsmState,
    ) -> Result<regs::EnsmState, EnsmError<Spi::Error>> {
        let current = regs::state::Register::new_with_raw_value(
            self.read_register(regs::Register::State).await?,
        );
        let current_state = current.ensm_state().map_err(EnsmError::InvalidEnsmState)?;
        if current_state == target_state {
            return Ok(current_state);
        }

        let mut config1 = regs::ensm_config_1::Register::new_with_raw_value(
            self.read_register(regs::Register::EnsmConfig1).await?,
        );
        if self.state.ensm_pin_control {
            config1.set_enable_ensm_pin_ctrl(false);
        }
        config1.set_to_alert(false);
        match target_state {
            regs::EnsmState::Alert => {
                config1.set_force_tx_on(false);
                config1.set_force_rx_on(false);
                config1.set_to_alert(true);
            }
            regs::EnsmState::Tx | regs::EnsmState::Fdd => {
                config1.set_force_tx_on(true);
            }
            regs::EnsmState::Rx => {
                config1.set_force_rx_on(true);
            }
            _ => {
                return Ok(current_state);
            }
        }

        self.inner
            .write_register(
                regs::Register::EnsmConfig1,
                regs::ensm_config_1::Register::ZERO
                    .with_to_alert(true)
                    .with_force_alert_state(true)
                    .raw_value(),
            )
            .await?;
        self.inner
            .write_register(regs::Register::EnsmConfig1, config1.raw_value())
            .await?;
        let mut current_ms_waited = 0;
        loop {
            let current = regs::state::Register::new_with_raw_value(
                self.read_register(regs::Register::State).await?,
            );
            let ensm_state = current.ensm_state();
            if let Ok(ensm_state) = ensm_state
                && ensm_state == target_state
            {
                break;
            }
            self.inner.delay.delay_ms(1).await;
            current_ms_waited += 1;
            if current_ms_waited > 10 {
                let value: u4 = match ensm_state {
                    Ok(state) => state.raw_value(),
                    Err(e) => e,
                };
                return Err(EnsmError::EnsmTransitionTimeout(value));
            }
        }

        Ok(current_state)
    }

    /// The driver's last-known ENSM (Enable State Machine) state, as tracked since
    /// [`Ad9361Uninit::init`] or the last call that changed it. This is a local cache, not a
    /// fresh register read.
    pub fn ensm_state(&self) -> EnsmState {
        self.state.current_ensm_state
    }

    /// The current clock tree configuration, as last applied by [`Ad9361Uninit::init`] or
    /// [`Ad9361::update_rf_clocks`]. This is a local cache reflecting what was actually written
    /// to the chip, not something callers should recompute themselves.
    pub fn clock_config(&self) -> &clocks::ClockConfig {
        &self.config.clock
    }

    /// Tunes the RX and TX digital interface delay, which are the `RX_CLOCK_DATA_DELAY`/
    /// `TX_CLOCK_DATA_DELAY` parameters, at the currently configured sample rate, returning the
    /// values that ended up applied.
    ///
    /// If no error-free window is found, this is not a hard error: the registers are left at
    /// whatever they held when this call started (read back and restored on the way out), and
    /// that value is returned. Genuine SPI/ENSM errors are always propagated.
    #[cfg(feature = "axi-tune")]
    pub async fn digital_tune(
        &mut self,
        adc: &mut axi_ad9361::adc::Adc,
        dac: &mut axi_ad9361::dac::Dac,
    ) -> Result<DigitalInterfaceDelay, TuneError<Spi::Error>> {
        let previous_delay = self.read_digital_interface_delay().await?;
        let previous_ensm_state = self.force_ensm_state(regs::EnsmState::Alert).await?;
        let previous_bist = self.read_register(regs::Register::BistConfig).await?;
        // Mirrors `phy->bist_loopback_mode`: `digital_tune_tx` puts the chip into internal
        // TX->RX loopback via this register and must not leave it there afterward.
        let previous_bist_config_2 = self.read_register(regs::Register::BistConfig2).await?;
        let mute_guard = self.mute_tx().await?;

        // Mirrors `ad9361_set_ensm_mode(phy, true, false)`: force both RX and TX synths active
        // for the duration of the tune (a TDD radio may otherwise time-share/power one of them
        // down between RX and TX). Unlike the C driver, this isn't gated on `!phy->pdata->fdd` —
        // saving and restoring the exact prior bytes makes it a no-op on an already-FDD radio,
        // so there's no need to know or track which mode is configured.
        let previous_ensm_mode = self.read_register(regs::Register::EnsmMode).await?;
        let previous_ensm_config_2 = self.read_register(regs::Register::EnsmConfig2).await?;
        self.write_register(
            regs::Register::EnsmMode,
            regs::ensm_mode::Register::ZERO
                .with_fdd_mode(true)
                .raw_value(),
        )
        .await?;
        // Matches the C driver's exact mask: clear fdd_external_ctrl_enable/txnrx_spi_ctrl/
        // synth_enable_pin_ctrl_mode (their interaction with dual_synth_mode isn't verified
        // here), keep power_down_*_synth/*_synth_ready_mask, force dual_synth_mode on.
        self.write_register(
            regs::Register::EnsmConfig2,
            regs::ensm_config_2::Register::new_with_raw_value(previous_ensm_config_2)
                .with_fdd_external_ctrl_enable(false)
                .with_txnrx_spi_ctrl(regs::ensm_config_2::TxNRxSpiCtrl::Rx)
                .with_synth_enable_pin_ctrl_mode(false)
                .with_dual_synth_mode(true)
                .raw_value(),
        )
        .await?;

        let result = match self.digital_tune_rx(adc).await {
            Ok(()) => self.digital_tune_tx(adc, dac).await,
            Err(e) => Err(e),
        };

        let new_digital_interface_delay = match result {
            Err(TuneError::NoValidWindow) => self
                .apply_digital_interface_delay(previous_delay)
                .await
                .map(|()| previous_delay)
                .map_err(TuneError::Spi),
            Err(e) => Err(e),
            Ok(()) => self
                .read_digital_interface_delay()
                .await
                .map_err(TuneError::Spi),
        };

        let _ = self
            .write_register(regs::Register::BistConfig, previous_bist)
            .await;
        let _ = self
            .write_register(regs::Register::BistConfig2, previous_bist_config_2)
            .await;
        let _ = self
            .write_register(regs::Register::EnsmConfig2, previous_ensm_config_2)
            .await;
        let _ = self
            .write_register(regs::Register::EnsmMode, previous_ensm_mode)
            .await;
        // Ignore errors for the clean-up handling.
        let _ = self.restore_ensm_state(previous_ensm_state).await;
        // A freshly-written (or, on NoValidWindow, restored) delay value doesn't reliably
        // relatch into the AXI core's digital interface without this pulse — mirrors the
        // unconditional `AXI_ADC_REG_RSTN` toggle at the very end of `ad9361_dig_tune()`,
        // separate from the one already done inside `digital_tune_rx()`.
        adc.reset_pulse();
        let _ = mute_guard.unmute(self).await;

        new_digital_interface_delay
    }

    /// Reloads a [`DigitalInterfaceDelay`] previously returned by [`Self::digital_tune`].
    #[cfg(feature = "axi-tune")]
    pub async fn restore_digital_tune(
        &mut self,
        adc: &mut axi_ad9361::adc::Adc,
        delay: DigitalInterfaceDelay,
    ) -> Result<(), TuneError<Spi::Error>> {
        let previous_ensm_state = self.force_ensm_state(regs::EnsmState::Alert).await?;
        let mute_guard = self.mute_tx().await?;

        let result = self
            .apply_digital_interface_delay(delay)
            .await
            .map_err(TuneError::Spi);

        let _ = self.restore_ensm_state(previous_ensm_state).await;
        adc.reset_pulse();
        let _ = mute_guard.unmute(self).await;

        result
    }

    /// Mirrors `ad9361_dig_tune_rx()`.
    #[cfg(feature = "axi-tune")]
    async fn digital_tune_rx(
        &mut self,
        adc: &mut axi_ad9361::adc::Adc,
    ) -> Result<(), TuneError<Spi::Error>> {
        // ad9361_bist_loopback(phy, 0): disable both the chip-internal and FPGA-side loopback.
        let bist_config_2 = regs::bist_config_2::Register::new_with_raw_value(
            self.read_register(regs::Register::BistConfig2).await?,
        )
        .with_data_port_sp_hd_loop_test_oe(false)
        .with_data_port_loop_test_enable(false);
        self.write_register(regs::Register::BistConfig2, bist_config_2.raw_value())
            .await?;

        // ad9361_bist_prbs(phy, BIST_INJ_RX): inject at ctrl point 2 (ADC digital output).
        let bist = regs::bist_config::Register::ZERO
            .with_bist_enable(true)
            .with_bist_ctrl_point(u2::new(2));
        self.write_register(regs::Register::BistConfig, bist.raw_value())
            .await?;

        let result = match self.calculate_interface_delay(adc, false).await {
            Ok(delay) => self.apply_interface_delay(false, delay).await,
            Err(e) => Err(e),
        };
        adc.reset_pulse();
        result
    }

    /// Mirrors `ad9361_dig_tune_tx()`.
    ///
    /// Modern-core-only (no `PCORE_VERSION_MAJOR < 8` fallback).
    #[cfg(feature = "axi-tune")]
    async fn digital_tune_tx(
        &mut self,
        adc: &mut axi_ad9361::adc::Adc,
        dac: &mut axi_ad9361::dac::Dac,
    ) -> Result<(), TuneError<Spi::Error>> {
        // ad9361_bist_prbs(phy, BIST_DISABLE)
        self.write_register(regs::Register::BistConfig, 0).await?;
        // ad9361_bist_loopback(phy, 1): AD9361-internal TX->RX loopback.
        self.set_internal_tx_rx_loopback().await?;
        dac.release_reset();

        let mut saved_adc_ctrl0 = [ChannelDataPathControl::ZERO; 4];
        let mut saved_dac_dsel = [ChannelDataSource::ZERO; 4];
        let mut saved_dac_ctrl6 = [ChannelLegacyControl::ZERO; 4];

        for ch in 0..self.config.num_axi_channels.as_usize() {
            let ch_as_u4 = u4::new(ch as u8);

            saved_adc_ctrl0[ch] = adc.channel_mut(ch_as_u4).read_data_path_control();
            adc.channel_mut(ch_as_u4).write_data_path_control(
                ChannelDataPathControl::ZERO
                    .with_format_signext(true)
                    .with_format_enable(true)
                    .with_enable(true)
                    .with_iqcor_enable(true),
            );
            adc.channel_mut(ch_as_u4)
                .modify_pn_select(|val| val.with_pn_sel(PnSel::PnCustom));

            saved_dac_ctrl6[ch] = dac.channel_mut(ch_as_u4).read_legacy_control();
            saved_dac_dsel[ch] = dac.channel_mut(ch_as_u4).read_data_source();
            dac.channel_mut(ch_as_u4)
                .write_data_source(ChannelDataSource::ZERO.with_data_source(DataSource::PnX));
            dac.channel_mut(ch_as_u4)
                .write_legacy_control(ChannelLegacyControl::ZERO);
            dac.synchronize();
        }

        let result = match self.calculate_interface_delay(adc, true).await {
            Ok(delay) => self.apply_interface_delay(true, delay).await,
            Err(e) => Err(e),
        };

        for ch in 0..self.config.num_axi_channels.as_usize() {
            let ch4 = u4::new(ch as u8);
            adc.channel_mut(ch4)
                .write_data_path_control(saved_adc_ctrl0[ch]);
            adc.channel_mut(ch4)
                .modify_pn_select(|val| val.with_pn_sel(PnSel::Pn9));
            dac.channel_mut(ch4).write_data_source(saved_dac_dsel[ch]);
            dac.synchronize();
            dac.channel_mut(ch4)
                .write_legacy_control(saved_dac_ctrl6[ch]);
        }

        result
    }

    /// Mirrors `ad9361_bist_loopback(phy, 1)`'s `REG_OBSERVE_CONFIG`/`BIST Config 2` bits,
    /// without `ad9361_int_loopback_fix_ch_cross()` (see [`Self::digital_tune`]'s docs).
    #[cfg(feature = "axi-tune")]
    async fn set_internal_tx_rx_loopback(&mut self) -> Result<(), Spi::Error> {
        let sp_hd = regs::parallel_port_conf_3::Register::new_with_raw_value(
            self.read_register(regs::Register::ParallelPortConf3)
                .await?,
        );
        let mut reg = regs::bist_config_2::Register::new_with_raw_value(
            self.read_register(regs::Register::BistConfig2).await?,
        );
        reg.set_data_port_sp_hd_loop_test_oe(
            sp_hd.single_port_mode()
                && sp_hd.duplex_mode() == regs::parallel_port_conf_3::Duplex::Half,
        );
        reg.set_data_port_loop_test_enable(true);
        self.write_register(regs::Register::BistConfig2, reg.raw_value())
            .await
    }

    /// Sweeps `RX_CLOCK_DATA_DELAY`/`TX_CLOCK_DATA_DELAY` and returns the widest error-free
    /// window found, without committing it. Mirrors the search half of `ad9361_dig_tune_delay()`.
    /// Note this necessarily writes each candidate delay to the register as it probes it —
    /// "calculate" can't be separated from register writes here, only from *committing* a final
    /// answer, which [`Self::apply_interface_delay`] does.
    #[cfg(feature = "axi-tune")]
    async fn calculate_interface_delay(
        &mut self,
        adc: &mut axi_ad9361::adc::Adc,
        tx: bool,
    ) -> Result<ClockDataDelay, TuneError<Spi::Error>> {
        // field[0]: clock delay held at 0, data delay swept 0..16.
        // field[1]: clock delay held at 15, data delay swept 15..0.
        let mut field = [[false; 16]; 2];
        for (i, row) in field.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                let (clock_delay, data_delay) = if i == 0 {
                    (0u8, j as u8)
                } else {
                    (15u8, 15 - j as u8)
                };
                self.set_interface_delay(
                    tx,
                    ClockDataDelay {
                        clock_delay: u4::new(clock_delay),
                        data_delay: u4::new(data_delay),
                    },
                    j == 0,
                )
                .await?;
                *cell = self.check_pn_error(adc, tx).await;
            }
        }

        let (count0, start0) = find_opt(&field[0]);
        let (count1, start1) = find_opt(&field[1]);
        if count0 == 0 && count1 == 0 {
            return Err(TuneError::NoValidWindow);
        }

        Ok(if count1 > count0 {
            ClockDataDelay {
                clock_delay: u4::new((start1 + count1 / 2) as u8),
                data_delay: u4::new(0),
            }
        } else {
            ClockDataDelay {
                clock_delay: u4::new(0),
                data_delay: u4::new((start0 + count0 / 2) as u8),
            }
        })
    }

    /// Commits a [`ClockDataDelay`] to the RX or TX clock-data-delay register.
    #[cfg(feature = "axi-tune")]
    async fn apply_interface_delay(
        &mut self,
        tx: bool,
        delay: ClockDataDelay,
    ) -> Result<(), TuneError<Spi::Error>> {
        self.set_interface_delay(tx, delay, true).await
    }

    /// Writes one clock/data delay pair. Mirrors `ad9361_set_intf_delay()`: when `clock_changed`,
    /// the ENSM is bounced Alert -> (write) -> FDD, which appears to be required for the new
    /// clock-delay value to actually latch into the digital interface hardware.
    #[cfg(feature = "axi-tune")]
    async fn set_interface_delay(
        &mut self,
        tx: bool,
        delay: ClockDataDelay,
        clock_changed: bool,
    ) -> Result<(), TuneError<Spi::Error>> {
        if clock_changed {
            self.force_ensm_state(regs::EnsmState::Alert).await?;
        }
        if tx {
            self.write_register(
                regs::Register::TxClockDataDelay,
                regs::tx_clock_data_delay::Register::ZERO
                    .with_fb_clk_delay(delay.clock_delay)
                    .with_tx_data_delay(delay.data_delay)
                    .raw_value(),
            )
            .await?;
        } else {
            self.write_register(
                regs::Register::RxClockDataDelay,
                regs::rx_clock_data_delay::Register::ZERO
                    .with_data_clk_delay(delay.clock_delay)
                    .with_rx_data_delay(delay.data_delay)
                    .raw_value(),
            )
            .await?;
        }
        if clock_changed {
            self.force_ensm_state(regs::EnsmState::Fdd).await?;
        }
        Ok(())
    }

    /// Clears each channel's sticky PN error/OOS bits, waits for the PN monitor to resettle,
    /// then reports whether any channel (or, for RX, the global lock bit) still shows an error.
    /// Mirrors `ad9361_check_pn()`.
    #[cfg(feature = "axi-tune")]
    async fn check_pn_error(&mut self, adc: &mut axi_ad9361::adc::Adc, tx: bool) -> bool {
        for ch in 0..self.config.num_axi_channels.as_usize() {
            adc.channel_mut(u4::new(ch as u8)).write_status(
                ChannelStatus::ZERO
                    .with_pn_error(true)
                    .with_pn_out_of_sync(true),
            );
        }
        self.inner.delay.delay_ms(4).await;

        if !tx && !adc.read_status().locked() {
            return true;
        }
        for ch in 0..self.config.num_axi_channels.as_usize() {
            let status = adc.channel_mut(u4::new(ch as u8)).read_status();
            // Mirrors the C driver's `if (adi_reg_chan_status) return 1;`: the whole raw
            // register is checked, not just the two bits we explicitly cleared above. On this
            // core, `status_header`/`crc_err` are hardwired to 0 (see `axi_ad9361_rx_channel.v`),
            // but `over_range` is a real, sticky bit that this function's own clear step doesn't
            // touch -- so it must be checked too, not just `pn_error`/`pn_out_of_sync`.
            //
            // NOTE: this is a faithful port of the C behavior, but it's not confirmed to matter
            // in practice -- a live A/B test (with vs. without this change) produced an
            // identical digital-tune sweep result both times, so `over_range` doesn't appear to
            // be the thing actually causing the RX/TX PN-check divergence under investigation.
            if status.raw_value() != 0 {
                return true;
            }
        }
        false
    }

    #[cfg(feature = "axi-tune")]
    async fn read_digital_interface_delay(&mut self) -> Result<DigitalInterfaceDelay, Spi::Error> {
        let rx = regs::rx_clock_data_delay::Register::new_with_raw_value(
            self.read_register(regs::Register::RxClockDataDelay).await?,
        );
        let tx = regs::tx_clock_data_delay::Register::new_with_raw_value(
            self.read_register(regs::Register::TxClockDataDelay).await?,
        );
        Ok(DigitalInterfaceDelay {
            rx: ClockDataDelay {
                clock_delay: rx.data_clk_delay(),
                data_delay: rx.rx_data_delay(),
            },
            tx: ClockDataDelay {
                clock_delay: tx.fb_clk_delay(),
                data_delay: tx.tx_data_delay(),
            },
        })
    }

    #[cfg(feature = "axi-tune")]
    async fn apply_digital_interface_delay(
        &mut self,
        delay: DigitalInterfaceDelay,
    ) -> Result<(), Spi::Error> {
        self.write_register(
            regs::Register::RxClockDataDelay,
            regs::rx_clock_data_delay::Register::ZERO
                .with_data_clk_delay(delay.rx.clock_delay)
                .with_rx_data_delay(delay.rx.data_delay)
                .raw_value(),
        )
        .await?;
        self.write_register(
            regs::Register::TxClockDataDelay,
            regs::tx_clock_data_delay::Register::ZERO
                .with_fb_clk_delay(delay.tx.clock_delay)
                .with_tx_data_delay(delay.tx.data_delay)
                .raw_value(),
        )
        .await?;
        Ok(())
    }
}

/// A clock/data delay pair for `RX_CLOCK_DATA_DELAY`/`TX_CLOCK_DATA_DELAY`. Internal to the
/// digital-tune sweep — named fields instead of a `(u4, u4)` tuple specifically so
/// `clock_delay`/`data_delay` can't be silently swapped when threaded through
/// `calculate_interface_delay` -> `apply_interface_delay`/`set_interface_delay`.
#[cfg(feature = "axi-tune")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct ClockDataDelay {
    clock_delay: u4,
    data_delay: u4,
}

/// Finds the widest run of `false` ("no error") entries in `field`, returning `(count, start)`.
/// Mirrors `ad9361_find_opt()` in the C driver.
#[cfg(feature = "axi-tune")]
fn find_opt(field: &[bool; 16]) -> (usize, usize) {
    let mut count = 0usize;
    let mut max_count = 0usize;
    let mut start: Option<usize> = None;
    let mut max_start = 0usize;
    for (i, &error) in field.iter().enumerate() {
        if !error {
            if start.is_none() {
                start = Some(i);
            }
            count += 1;
        } else {
            if count > max_count {
                max_count = count;
                max_start = start.unwrap_or(0);
            }
            start = None;
            count = 0;
        }
    }
    if count > max_count {
        max_count = count;
        max_start = start.unwrap_or(0);
    }
    (max_count, max_start)
}

#[cfg(all(test, feature = "axi-tune"))]
mod find_opt_tests {
    extern crate std;
    use super::find_opt;

    /// `f` = false (no error), `t` = true (error), for compact test tables.
    const F: bool = false;
    const T: bool = true;

    #[test]
    fn all_clean_is_one_full_length_window() {
        assert_eq!(find_opt(&[F; 16]), (16, 0));
    }

    #[test]
    fn all_errors_has_no_window() {
        assert_eq!(find_opt(&[T; 16]), (0, 0));
    }

    #[test]
    fn single_run_in_the_middle() {
        let field = [T, T, F, F, F, F, T, T, T, T, T, T, T, T, T, T];
        assert_eq!(find_opt(&field), (4, 2));
    }

    #[test]
    fn picks_the_longer_of_two_runs() {
        let field = [F, F, T, F, F, F, F, T, T, T, T, T, T, T, T, T];
        assert_eq!(find_opt(&field), (4, 3));
    }

    #[test]
    fn ties_keep_the_first_run() {
        // Two equal-length runs of 3 (indices 0-2 and 6-8): C's `if (cnt > max_cnt)` is a
        // strict greater-than, so on a tie the earlier run wins, not the later one.
        let field = [F, F, F, T, T, T, F, F, F, T, T, T, T, T, T, T];
        assert_eq!(find_opt(&field), (3, 0));
    }

    #[test]
    fn run_touching_the_start() {
        let field = [F, F, F, F, F, T, T, T, T, T, T, T, T, T, T, T];
        assert_eq!(find_opt(&field), (5, 0));
    }

    #[test]
    fn run_touching_the_end() {
        // Exercises the post-loop flush: the trailing run never hits an `error` sample to
        // trigger the `else` branch's max-update, so it only gets counted after the loop ends.
        let field = [T, T, T, T, T, T, T, T, T, T, T, F, F, F, F, F];
        assert_eq!(find_opt(&field), (5, 11));
    }

    #[test]
    fn single_isolated_clean_sample() {
        let mut field = [T; 16];
        field[7] = F;
        assert_eq!(find_opt(&field), (1, 7));
    }
}

/// An RX/TX digital interface delay (`RX_CLOCK_DATA_DELAY`/`TX_CLOCK_DATA_DELAY`) pair, as
/// returned by [`Ad9361::digital_tune`] and accepted by [`Ad9361::restore_digital_tune`]. Not
/// meant to be constructed or inspected by callers, just held and passed back.
#[cfg(feature = "axi-tune")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DigitalInterfaceDelay {
    rx: ClockDataDelay,
    tx: ClockDataDelay,
}

/// Error from [`Ad9361::digital_tune`]/[`Ad9361::restore_digital_tune`].
#[cfg(feature = "axi-tune")]
#[derive(Debug, thiserror::Error)]
pub enum TuneError<Spi> {
    /// SPI error.
    #[error("SPI error: {0}")]
    Spi(#[from] Spi),
    /// ENSM error.
    #[error("ENSM error: {0}")]
    Ensm(#[from] EnsmError<Spi>),
    /// No interface delay window without errors was found.
    #[error("no error-free interface delay window found")]
    NoValidWindow,
    /// Tuning across the fixed calibration rates is not implemented yet.
    #[error("tuning across the fixed calibration rates (max_freq) is not implemented yet")]
    RateSweepUnsupported,
}

/// Error while reading or changing the ENSM state.
#[derive(Debug, thiserror::Error)]
pub enum EnsmError<Spi> {
    /// SPI error.
    #[error("SPI error: {0}")]
    SpiError(#[from] Spi),
    /// The state register holds an invalid ENSM state. The value is the raw register value.
    #[error("Invalid ENSM state: {0}")]
    InvalidEnsmState(u4),
    /// The state transition timed out. The value is the last raw state read.
    #[error("Timeout waiting for ENSM state transition")]
    EnsmTransitionTimeout(u4),
}

const fn div_round_u32(a: u32, b: u32) -> u32 {
    (a + b / 2) / b
}

const fn div_round_u64(a: u64, b: u64) -> u64 {
    (a + b / 2) / b
}

/// C-compatible port of `ad9361_find_opt` for boolean failure masks.
///
/// `failed[i] == true` means the phase failed; this returns the start and length of the longest
/// contiguous run of successful entries (`false`).
fn find_longest_success_window(failed: &[bool]) -> (usize, usize) {
    let mut current_start: Option<usize> = None;
    let mut current_len: usize = 0;

    let mut best_start: usize = 0;
    let mut best_len: usize = 0;

    for (idx, did_fail) in failed.iter().copied().enumerate() {
        if !did_fail {
            if current_start.is_none() {
                current_start = Some(idx);
            }
            current_len += 1;
        } else {
            if current_len > best_len {
                best_len = current_len;
                best_start = current_start.unwrap_or(0);
            }
            current_start = None;
            current_len = 0;
        }
    }

    if current_len > best_len {
        best_len = current_len;
        best_start = current_start.unwrap_or(0);
    }

    (best_start, best_len)
}
