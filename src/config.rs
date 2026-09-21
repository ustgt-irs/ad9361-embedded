use core::num::NonZero;

pub use arbitrary_int::{u2, u3, u4, u5, u6, u7, u10, u13};

pub use regs::lvds_bias_ctrl::Register as BiasControlRegister;
pub use regs::lvds_invert_ctrl1::Register as LvdsInversionRegister1;
pub use regs::lvds_invert_ctrl2::Register as LvdsInversionRegister2;
pub use regs::parallel_port_conf_1::Register as Pp1Register;
pub use regs::parallel_port_conf_2::Register as Pp2Register;
pub use regs::rx_clock_data_delay::Register as RxClockDelayConfig;
pub use regs::tx_clock_data_delay::Register as TxClockDelayConfig;

pub use regs::rssi_config::RestartMode as RssiRestartMode;

use crate::limits::MAX_TX_ATTENUATION_DB;
use crate::regs::ClkOutMode;
pub use crate::regs::input_select::{RxRfPathConfig, TxRfPathConfig};
use crate::types::{
    FastAgcTargetGainIndexType, GainControlMode, ReceiverId, RxClock, TransmitterId, TxClock,
};
use crate::{clocks, limits, regs};
pub use regs::DecPowerMeasurementSource;
pub use regs::parallel_port_conf_3::DataRate;

pub use super::clocks::ClockConfig;
pub use crate::lut::RxGainTableType;

/// Kind of gain step size which failed the validation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StepSizeKind {
    /// Increment step of the manual gain control.
    ManualGainInc,
    /// Decrement step of the manual gain control.
    ManualGainDec,
    /// Step size of the digital gain.
    DigitalGain,
}

/// Error while validating a [`ConfigRaw`].
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// The RX RF bandwidth is out of range.
    #[error("Invalid RF RX bandwidth")]
    InvalidRfRxBandwidth,
    /// The TX RF bandwidth is out of range.
    #[error("Invalid RF TX bandwidth")]
    InvalidRfTxBandwidth,
    /// The CMOS data clock is larger than 61.44 MSPS.
    #[error("Invalid CMOS data clock larger than 61.44 MSPS")]
    InvalidCmosDataClock,
    /// An RX clock rate is out of range.
    #[error("Invalid RX clock rate")]
    InvalidRxClock(RxClock),
    /// A TX clock rate is out of range.
    #[error("Invalid TX clock rate")]
    InvalidTxClock(TxClock),
    /// The ADC overrange sample size is not in the range 1 to 8.
    #[error("Invalid ADC overrange sample, must be in range 1..=8")]
    AdcOverrangeSampleInvalid(u8),
    /// A gain step size is not in the range 1 to 8.
    #[error("Invalid step size for {0:?}, value: {1}")]
    InvalidStepSize(StepSizeKind, u8),
    /// The small ADC overload threshold is larger than the large one.
    #[error("invalid ADC thresholds")]
    InvalidAdcOverloadThresholds,
    /// The low LMT overload threshold is larger than the high one.
    #[error("invalid LMT overload thresholds")]
    InvalidLmtOverloadThreshold,
    /// The inner low AGC threshold is not lower than the inner high threshold.
    #[error("invalid AGC thresholds")]
    InvalidAgcThresholds,
    /// The TX attenuation is larger than the maximum.
    #[error("TX attenuation invalid, out of bounds")]
    TxAttenuationInvalid(u32),
}

/// Channels a FIR filter is applied to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirChSelect {
    /// Channel 1.
    Ch1,
    /// Channel 2.
    Ch2,
    /// Both channels.
    Both,
}

impl FirChSelect {
    /// Returns true if channel 1 is selected.
    pub const fn channel1(&self) -> bool {
        matches!(self, FirChSelect::Ch1 | FirChSelect::Both)
    }

    /// Returns true if channel 2 is selected.
    pub const fn channel2(&self) -> bool {
        matches!(self, FirChSelect::Ch2 | FirChSelect::Both)
    }
}

/// TX FIR filter configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxFirConfig<'coef> {
    /// Channels the filter is applied to.
    pub ch_select: FirChSelect,
    /// -6dB gain.
    pub attentuate_6db: bool,
    /// Coefficient for each taps. Only a part of this might be used, depending on the number
    /// of taps.
    pub tx_coefs: &'coef [i16],
    /// Number of taps.
    pub num_taps: regs::tx_filter_conf::NumberOfTapsRaw,
}

/// The number of coefficients does not match the number of taps.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid number of coefficients")]
pub struct InvalidNumberOfCoefficientsError;

impl TxFirConfig<'_> {
    /// Returns true if the number of coefficients equals the number of taps.
    pub const fn tx_coeffs_size_equal_to_num_of_taps(&self) -> bool {
        self.tx_coefs.len() == self.num_taps.number() as usize
    }
}

/// RX FIR filter configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RxFirConfig<'coef> {
    /// Channels the filter is applied to.
    pub ch_select: FirChSelect,
    /// Gain of the filter.
    pub gain: regs::rx_filter_gain::FilterGain,
    /// Coefficient for each taps. Only a part of this might be used, depending on the number
    /// of taps.
    pub rx_coefs: &'coef [i16],
    /// Number of taps.
    pub num_taps: regs::tx_filter_conf::NumberOfTapsRaw,
}

impl RxFirConfig<'_> {
    /// Returns true if the number of coefficients equals the number of taps.
    pub const fn rx_coeffs_size_equal_to_num_of_taps(&self) -> bool {
        self.rx_coefs.len() == self.num_taps.number() as usize
    }
}

/// Select between enabling all channels and only enabling one TX and one RX path.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TransceiverChannelMode {
    /// All channels are enabled.
    Dual,
    /// Only one TX and one RX channel are enabled.
    Single {
        /// Enabled RX channel.
        rx: ReceiverId,
        /// Enabled TX channel.
        tx: TransmitterId,
    },
}

/// Configuration of the parallel port mode register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelPortModeConfig(pub regs::parallel_port_conf_3::Register);

impl ParallelPortModeConfig {
    /// Configuration for the LVDS interface.
    pub const fn new_for_lvds() -> Self {
        Self(
            regs::parallel_port_conf_3::Register::builder()
                .with_fdd_rx_rate_2tx_rate(false)
                .with_swap_ports(false)
                .with_data_rate(regs::parallel_port_conf_3::DataRate::Double)
                .with_lvds_mode(true)
                .with_duplex_mode(regs::parallel_port_conf_3::Duplex::Full)
                .with_single_port_mode(false)
                .with_full_port(false)
                .with_full_duplex_swap_bits(false)
                .build(),
        )
    }

    /// Configuration for the CMOS interface with two ports in full duplex mode.
    pub const fn new_for_dual_port_full_duplex_cmos(
        data_rate: DataRate,
        rx_is_tx_times_two: bool,
        swap_ports: bool,
        swap_bits: bool,
    ) -> Self {
        Self(
            regs::parallel_port_conf_3::Register::builder()
                .with_fdd_rx_rate_2tx_rate(rx_is_tx_times_two)
                .with_swap_ports(swap_ports)
                .with_data_rate(data_rate)
                .with_lvds_mode(false)
                .with_duplex_mode(regs::parallel_port_conf_3::Duplex::Full)
                .with_single_port_mode(false)
                .with_full_port(true)
                .with_full_duplex_swap_bits(swap_bits)
                .build(),
        )
    }

    /// Configuration for the CMOS interface with two ports in half duplex mode.
    pub const fn new_for_dual_port_half_duplex_cmos(data_rate: DataRate) -> Self {
        Self(
            regs::parallel_port_conf_3::Register::builder()
                .with_fdd_rx_rate_2tx_rate(false)
                .with_swap_ports(false)
                .with_data_rate(data_rate)
                .with_lvds_mode(false)
                .with_duplex_mode(regs::parallel_port_conf_3::Duplex::Half)
                .with_single_port_mode(false)
                .with_full_port(false)
                .with_full_duplex_swap_bits(false)
                .build(),
        )
    }

    /// Configuration for the CMOS interface with one port in half duplex mode.
    pub const fn new_for_single_port_half_duplex_cmos(data_rate: DataRate) -> Self {
        Self(
            regs::parallel_port_conf_3::Register::builder()
                .with_fdd_rx_rate_2tx_rate(false)
                .with_swap_ports(false)
                .with_data_rate(data_rate)
                .with_lvds_mode(false)
                .with_duplex_mode(regs::parallel_port_conf_3::Duplex::Half)
                .with_single_port_mode(true)
                .with_full_port(true)
                .with_full_duplex_swap_bits(false)
                .build(),
        )
    }
}

/// Source of the reference clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExternalClockConfig {
    /// External oscillator connected to the REF_CLK_IN pin.
    Oscillator,
    /// External crystal in combination with internal digital programmable capacitor.
    Dcxo {
        /// Coarse tune value of the DCXO.
        coarse_tune: u6,
        /// Fine tune value of the DCXO.
        fine_tune: u13,
    },
}

impl From<ExternalClockConfig> for regs::ExternalClockConfig {
    fn from(value: ExternalClockConfig) -> Self {
        match value {
            ExternalClockConfig::Oscillator => Self::Oscillator,
            ExternalClockConfig::Dcxo { .. } => Self::Dcxo,
        }
    }
}

/// Configuration of the TX monitor.
#[derive(Debug, Clone)]
pub enum TxMonitorConfig {
    /// The TX monitor is enabled with the given parameters.
    Enabled(TxMonConfigParams),
    /// The TX monitor is disabled.
    Disabled,
}

/// Driver configuration which is not validated yet. Use [`ConfigRaw::validate`] to get a
/// [`ConfigValidated`].
#[derive(Debug, Clone)]
pub struct ConfigRaw {
    // Clock configuration.
    /// Rate of the reference clock in Hz.
    pub reference_clk_rate: u32,

    /// RX bandwidth.
    pub rf_rx_bandwidth_hz: u32,
    /// TX bandwidth.
    pub rf_tx_bandwidth_hz: u32,

    /// Clock configuration.
    ///
    /// If `clock.rx.rx_fir`/`clock.tx.tx_fir` are set to a non-bypass value, `init()` alone is
    /// not sufficient for correct operation. See the doc comments on those fields and on
    /// [`crate::Ad9361Uninit::init`] for why a follow-up call to load real filter taps is
    /// required.
    pub clock: clocks::ClockConfig,
    /// Transceiver channel mode.
    pub transceiver_channel_mode: TransceiverChannelMode,
    /// Hardware configuration of the RX RF path.
    pub rx_path: RxRfPathConfig,
    /// Hardware configuration of the TX RF path.
    pub tx_path: TxRfPathConfig,

    /// TX monitor configuration.
    pub tx_monitoring: TxMonitorConfig,

    /// Division duplex configuration, FDD or TDD.
    pub division_duplex_config: DivisionDuplexConfig,

    /// Digital Interface Control
    pub digital_interface: DigitalInterfaceConfig,

    /// Source of the reference clock.
    pub external_clock_config: ExternalClockConfig,

    /// RX gain table type.
    pub gain_table_type: RxGainTableType,

    /// If [None], the CLKOUT pin is driven to 0.
    pub clkout_mode: Option<ClkOutMode>,

    // ENSM config
    /// Use pulse mode instead of level mode for the ENABLE pin.
    pub ensm_enable_pin_pulse_mode: bool,
    /// Control the ENSM with the ENABLE and TXNRX pins.
    pub ensm_enable_txnrx_control: bool,

    /// TX quadrature calibration. If [`None`], the calibration is skipped.
    pub tx_quad_calib: Option<crate::types::RxPhaseConfig>,

    /// Events which update the RF DC offset tracking.
    pub dc_offset_tracking_update_event: u3,
    /// RF DC offset attenuation for the high gain range.
    pub dc_offset_attenuation_high_range: u8,
    /// RF DC offset attenuation for the low gain range.
    pub dc_offset_attenuation_low_range: u8,
    /// RF DC offset count for the high gain range.
    pub dc_offset_count_high_range: u8,
    /// RF DC offset count for the low gain range.
    pub dc_offset_count_low_range: u8,
    /// Enables the slow mode of the QEC tracking.
    pub qec_tracking_slow_mode: bool,

    // TX Attenuation Control
    /// TX attenuation in milli-dB.
    pub tx_attenuation_md_b: u32,
    /// Update the TX gain in the ALERT state.
    pub update_tx_gain_in_alert: bool,

    /// Gain control.
    pub gain_control: GainControl,
    /// RSSI control
    pub rssi: RssiConfig,
    /// AuxDAC config
    pub aux_adc: AuxAdcConfig,
    /// AuxDAC config
    pub aux_dac: AuxDacConfig,

    // Control Out Setup
    /// Mask of the enabled control output pins.
    pub ctrl_outs_enable_mask: u8,
    /// Index of the control output signal group.
    pub ctrl_outs_index: u8,

    /// External LNA Control
    pub elna: Option<ExternalLnaConfig>,

    /// GPO pin configuration.
    pub gpo_config: GpoConfig,
}

impl ConfigRaw {
    /// Validates the configuration.
    pub fn validate(self) -> Result<ConfigValidated, ValidationError> {
        if self.rf_rx_bandwidth_hz < limits::MIN_RF_BW
            || self.rf_rx_bandwidth_hz > limits::MAX_RF_BW
        {
            return Err(ValidationError::InvalidRfRxBandwidth);
        }
        if self.rf_tx_bandwidth_hz < super::limits::MIN_RF_BW
            || self.rf_tx_bandwidth_hz > limits::MAX_RF_BW
        {
            return Err(ValidationError::InvalidRfTxBandwidth);
        }
        if self.gain_control.common.adc_ovr_sample_size == 0
            || self.gain_control.common.adc_ovr_sample_size > 8
        {
            return Err(ValidationError::AdcOverrangeSampleInvalid(
                self.gain_control.common.adc_ovr_sample_size,
            ));
        }
        if let Some(manual_config) = self.gain_control.manual() {
            if manual_config.inc_gain_step == 0 || manual_config.inc_gain_step > 8 {
                return Err(ValidationError::InvalidStepSize(
                    StepSizeKind::ManualGainInc,
                    manual_config.inc_gain_step,
                ));
            }
            if manual_config.dec_gain_step == 0 || manual_config.dec_gain_step > 8 {
                return Err(ValidationError::InvalidStepSize(
                    StepSizeKind::ManualGainDec,
                    manual_config.dec_gain_step,
                ));
            }
        }
        if let Some(digital_config) = &self.gain_control.common.digital_config
            && (digital_config.dig_gain_step_size == 0 || digital_config.dig_gain_step_size > 8)
        {
            return Err(ValidationError::InvalidStepSize(
                StepSizeKind::DigitalGain,
                digital_config.dig_gain_step_size,
            ));
        }
        if self.gain_control.common.adc_small_overload_thresh
            > self.gain_control.common.adc_large_overload_thresh
        {
            return Err(ValidationError::InvalidAdcOverloadThresholds);
        }
        if self.gain_control.common.lmt_overload_low_thresh_mv_peak
            > self.gain_control.common.lmt_overload_high_thresh_mv_peak
        {
            return Err(ValidationError::InvalidAdcOverloadThresholds);
        }
        if let Some(auto_config) = self.gain_control.auto()
            && auto_config.inner_thresh_low_negative_dbfs
                < auto_config.inner_thresh_high_negative_dbfs
        {
            return Err(ValidationError::InvalidAgcThresholds);
        }
        if self.tx_attenuation_md_b > MAX_TX_ATTENUATION_DB {
            return Err(ValidationError::TxAttenuationInvalid(
                self.tx_attenuation_md_b,
            ));
        }

        Ok(ConfigValidated(self))
    }
}

/// Configuration which passed [`ConfigRaw::validate`].
pub struct ConfigValidated(pub ConfigRaw);

/// Duplex mode of the transceiver.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DivisionDuplexMode {
    /// Frequency division duplex mode. In this mode, RX and TX can work at the same time on
    /// different frequency bands.
    Fdd,
    /// Time division duplex mode. In this mode, the RX and TX synthesizers are shared, and the
    /// device rapidly switches between RX and TX modes.
    Tdd,
}

/// Duplex mode configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DivisionDuplexConfig {
    /// Frequency division duplex.
    Fdd {
        /// Enables the independent mode, which sets the FDD external control enable.
        independent_mode: bool,
    },
    /// Time division duplex.
    Tdd {
        /// Enables the TX monitor.
        enable_txmon: bool,
        /// Use both the RX and TX synthesizer instead of sharing one.
        use_dual_synth: bool,
    },
}

impl DivisionDuplexConfig {
    /// Returns the duplex mode.
    pub const fn mode(&self) -> DivisionDuplexMode {
        match self {
            DivisionDuplexConfig::Fdd { .. } => DivisionDuplexMode::Fdd,
            DivisionDuplexConfig::Tdd { .. } => DivisionDuplexMode::Tdd,
        }
    }
}

/// Auxiliary DAC configuration.
#[derive(Debug, Clone)]
pub struct AuxDacConfig {
    /// Enables the manual mode of the auxiliary DACs.
    pub aux_dac_manual_mode_enable: bool,
    /// Configuration of DAC 1 and DAC 2. A DAC which is [`None`] is left unconfigured.
    pub dac_config: [Option<SingleDacConfig>; 2],
}

/// TX monitor parameters.
#[derive(Debug, Clone)]
pub struct TxMonConfigParams {
    /// Enable TX1 monitor.
    pub enable_tx1: bool,
    /// Enable TX2 monitor.
    pub enable_tx2: bool,

    /// Threshold between the low and the high gain in milli-dB.
    pub low_high_gain_threshold_md_b: u32,
    /// Low gain in dB.
    pub low_gain_d_b: u5,
    /// High gain in dB.
    pub high_gain_d_b: u5,
    /// Enables the TX monitor tracking.
    pub tx_mon_track: bool,
    /// Enables the one shot mode.
    pub one_shot_mode: bool,
    /// Delay of the TX monitor measurement.
    pub tx_mon_delay: u10,
    /// Duration of the TX monitor measurement.
    pub tx_mon_duration: u32,
    /// Front end gain of the TX1 monitor.
    pub tx1_mon_front_end_gain: u2,
    /// Front end gain of the TX2 monitor.
    pub tx2_mon_front_end_gain: u2,
    /// LO common mode of the TX1 monitor.
    pub tx1_mon_lo_cm: u6,
    /// LO common mode of the TX2 monitor.
    pub tx2_mon_lo_cm: u6,
}

/// Configuration of a single auxiliary DAC.
#[derive(Debug, Clone)]
pub struct SingleDacConfig {
    /// Default output voltage in mV.
    pub default_value_m_v: u32,

    /// The DAC is active in the RX state.
    pub active_in_rx: bool,
    /// The DAC is active in the TX state.
    pub active_in_tx: bool,
    /// The DAC is active in the ALERT state.
    pub active_in_alert: bool,

    /// Delay in us before the DAC output is applied after entering the RX state.
    pub rx_delay_us: u8,
    /// Delay in us before the DAC output is applied after entering the TX state.
    pub tx_delay_us: u8,
}

/// General Purpose Output (GPO) pin configuration.
#[derive(Debug, Clone)]
pub struct GpoConfig {
    /// When set, the GPOs can be controlled manually. Otherwise, the GPOs are ENSM slaves.
    pub manual_mode_enable: bool,
    /// Configuration of GPO 0 to 3.
    pub gpo: [SingleGpoConfig; 4],
}

impl Default for GpoConfig {
    fn default() -> Self {
        Self {
            manual_mode_enable: true,
            gpo: [SingleGpoConfig {
                init_state: false,
                inactive_state_high: false,
                slave_rx_enable: false,
                slave_tx_enable: false,
                rx_delay_us: 0,
                tx_delay_us: 0,
            }; 4],
        }
    }
}

/// Configuration for an individual pin.
#[derive(Debug, Default, Copy, Clone)]
pub struct SingleGpoConfig {
    /// Initial state of the pin.
    pub init_state: bool,
    /// The pin is high while it is inactive.
    pub inactive_state_high: bool,
    /// The pin follows the ENSM as a slave in the RX state.
    pub slave_rx_enable: bool,
    /// The pin follows the ENSM as a slave in the TX state.
    pub slave_tx_enable: bool,
    /// Delay in us after entering the RX state.
    pub rx_delay_us: u8,
    /// Delay in us after entering the TX state.
    pub tx_delay_us: u8,
}

/// Digital interface configuration.
#[derive(Debug, Clone)]
pub struct DigitalInterfaceConfig {
    /// Parallel port register 1.
    pub pp1_config: Pp1Register,

    /// Parallel port register 2.
    pub pp2_config: Pp2Register,

    /// Parallel port mode configuration, which is register 3.
    pub pp3_config: ParallelPortModeConfig,

    /// RX interface delay, written to `RX_CLOCK_DATA_DELAY` at init.
    ///
    /// Without the `axi-tune` feature, this is the delay, permanently. With it, this is just the
    /// starting point: [`crate::Ad9361::digital_tune`] may later calculate and apply a different
    /// value.
    pub rx_default_delay: RxClockDelayConfig,
    /// TX interface delay, written to `TX_CLOCK_DATA_DELAY` at init. See
    /// [`Self::rx_default_delay`].
    pub tx_default_delay: TxClockDelayConfig,

    /// LVDS configuration. Not used if [`None`].
    pub lvds_config: Option<LvdsConfig>,
    /// Enables the phase inversion of RX1 and RX2.
    pub rx1rx2_phase_inversion: bool,
}

/// LVDS interface configuration.
#[derive(Debug, Clone)]
pub struct LvdsConfig {
    /// Bias control register.
    pub bias_control: BiasControlRegister,
    /// Inversion register 1.
    pub invert1_control: LvdsInversionRegister1,
    /// Inversion register 2.
    pub invert2_control: LvdsInversionRegister2,
}

impl LvdsConfig {
    /// Creates the configuration for an LVDS bias voltage in mV and the on-chip RX termination.
    pub fn new(lvds_bias_mv: u32, rx_onchip_termination: bool) -> Self {
        Self {
            bias_control: BiasControlRegister::ZERO
                .with_lvds_bias(Self::lvds_bias_reg_value(lvds_bias_mv))
                .with_rx_on_chip_term(rx_onchip_termination),
            invert1_control: LvdsInversionRegister1::new_with_raw_value(0xFF),
            invert2_control: LvdsInversionRegister2::new_with_raw_value(0x0F),
        }
    }

    /// Returns the register value for an LVDS bias voltage in mV.
    pub const fn lvds_bias_reg_value(milli_volts: u32) -> u3 {
        u3::new(((milli_volts.saturating_sub(75) / 75) & 0x7) as u8)
    }
}

impl DigitalInterfaceConfig {
    /// Parallel port register 1.
    pub const fn pp1_config(&self) -> regs::parallel_port_conf_1::Register {
        self.pp1_config
    }

    /// Parallel port register 2.
    pub const fn pp2_config(&self) -> regs::parallel_port_conf_2::Register {
        self.pp2_config
    }

    /// Parallel port register 3.
    pub const fn pp3_config(&self) -> regs::parallel_port_conf_3::Register {
        self.pp3_config.0
    }
}

/// External LNA configuration.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExternalLnaConfig {
    /// Settling delay of the external LNA in ns.
    pub settling_delay_ns: u32,
    /// Gain of the external LNA in milli-dB.
    pub gain_md_b: u32,
    /// Loss of the external LNA bypass in milli-dB.
    pub bypass_loss_md_b: u32,
    /// The external LNA of RX1 is controlled by GPO0.
    pub rx1_gpo0_control: bool,
    /// The external LNA of RX2 is controlled by GPO1.
    pub rx2_gpo1_control: bool,
    /// Enable the external LNA for all indexes of the gain table.
    pub enable_for_all_indexes_in_gt: bool,
}

/// Error while creating a [`GainControl`].
#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum GainControlConfigError {
    /// The manual configuration is missing.
    #[error("manual config required when either channel is in manual mode")]
    ManualConfigMissing,
    /// The auto configuration is missing.
    #[error("auto config required when either channel uses an auto AGC mode")]
    AutoConfigMissing,
    /// The fast AGC configuration is missing.
    #[error("fast AGC config required when either channel uses fast or hybrid AGC mode")]
    FastConfigMissing,
}

/// Gain control configuration.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GainControl {
    rx1_mode: GainControlMode,
    rx2_mode: GainControlMode,
    manual: Option<GainControlManualConfig>,
    auto: Option<GainControlAutoConfig>,
    fast: Option<GainControlFastAutoConfig>,
    /// Configuration shared by all gain control modes.
    pub common: GainControlCommon,
}

impl GainControl {
    /// Creates a new gain control configuration. The configurations of all modes used by the
    /// channels must be provided.
    pub fn new(
        rx1_mode: GainControlMode,
        rx2_mode: GainControlMode,
        common: GainControlCommon,
        manual: Option<GainControlManualConfig>,
        auto: Option<GainControlAutoConfig>,
        fast: Option<GainControlFastAutoConfig>,
    ) -> Result<Self, GainControlConfigError> {
        let needs_manual =
            rx1_mode == GainControlMode::Manual || rx2_mode == GainControlMode::Manual;
        let needs_auto = matches!(
            rx1_mode,
            GainControlMode::AutoSlowAttack
                | GainControlMode::AutoFastAttack
                | GainControlMode::AutoHybrid
        ) || matches!(
            rx2_mode,
            GainControlMode::AutoSlowAttack
                | GainControlMode::AutoFastAttack
                | GainControlMode::AutoHybrid
        );
        let needs_fast = matches!(
            rx1_mode,
            GainControlMode::AutoFastAttack | GainControlMode::AutoHybrid
        ) || matches!(
            rx2_mode,
            GainControlMode::AutoFastAttack | GainControlMode::AutoHybrid
        );

        if needs_manual && manual.is_none() {
            return Err(GainControlConfigError::ManualConfigMissing);
        }
        if needs_auto && auto.is_none() {
            return Err(GainControlConfigError::AutoConfigMissing);
        }
        if needs_fast && fast.is_none() {
            return Err(GainControlConfigError::FastConfigMissing);
        }

        Ok(Self {
            rx1_mode,
            rx2_mode,
            common,
            manual,
            auto,
            fast,
        })
    }

    /// Gain control mode of RX1.
    #[inline]
    pub const fn rx1_mode(&self) -> GainControlMode {
        self.rx1_mode
    }

    /// Gain control mode of RX2.
    #[inline]
    pub const fn rx2_mode(&self) -> GainControlMode {
        self.rx2_mode
    }

    /// Mutable manual gain control configuration.
    #[inline]
    pub const fn manual_mut(&mut self) -> Option<&mut GainControlManualConfig> {
        self.manual.as_mut()
    }

    /// Mutable auto gain control configuration.
    #[inline]
    pub const fn auto_mut(&mut self) -> Option<&mut GainControlAutoConfig> {
        self.auto.as_mut()
    }

    /// Mutable fast AGC configuration.
    #[inline]
    pub const fn fast_mut(&mut self) -> Option<&mut GainControlFastAutoConfig> {
        self.fast.as_mut()
    }

    /// Manual gain control configuration.
    #[inline]
    pub const fn manual(&self) -> Option<&GainControlManualConfig> {
        self.manual.as_ref()
    }

    /// Auto gain control configuration.
    #[inline]
    pub const fn auto(&self) -> Option<&GainControlAutoConfig> {
        self.auto.as_ref()
    }

    /// Fast AGC configuration.
    #[inline]
    pub const fn fast(&self) -> Option<&GainControlFastAutoConfig> {
        self.fast.as_ref()
    }
}

/// Digital gain configuration.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct DigitalGainConfig {
    /// Maximum digital gain.
    pub max_dig_gain: u5,
    /// Must be in range 1..=8.
    pub dig_gain_step_size: u8,
}

/// Raw register value wrapper.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct DecPowerMeasurementDuration(pub u4);

impl DecPowerMeasurementDuration {
    /// Creates the duration from the register value.
    pub const fn new(value: u4) -> Self {
        Self(value)
    }

    /// Corresponding RX cycles.
    pub const fn rx_cycles(&self) -> u32 {
        16 << self.0.value() as u32
    }
}
/// Gain control configuration shared by all modes.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GainControlCommon {
    /// Digital gain configuration. The digital gain is not used if [`None`].
    pub digital_config: Option<DigitalGainConfig>,
    /// Must be non-zero and not larger than 8.
    pub adc_ovr_sample_size: u8,
    /// Power measurement duratin in RX cycles. The formula for the duration
    /// is 16 * 2 to the power of a 4 bit regsiter value. The driver will calculate
    /// the closest valid value using ilog2. Thus, the minimum value in RX samples is 16.
    pub dec_pow_measurement_duration: DecPowerMeasurementDuration,
    /// Used by fast AGC to determine if the gain should be increased if D0 of register 0x110
    /// is set. It can also be used in manual mode to trigger a CTRL_OUT signal transition.
    ///
    /// Units are negative dBFS, resolution is 0.5 dB/LSB. The range of this value is
    /// 0 to 64.
    pub low_power_thresh: u8,
    /// Signal used for the decimated power measurement.
    pub dec_pwr_meas_source: DecPowerMeasurementSource,
    // Not sure if this belongs here, but we try to remain close to the C lib for now.
    /// Interval between gain updates in us.
    pub gain_update_interval_us: u32,
    // These can also be used in manual gain control mode.
    /// Look at the datasheet p.36 for more information how to calculate those.
    pub adc_small_overload_thresh: u8,
    /// Look at the datasheet p.36 for more information how to calculate those.
    pub adc_large_overload_thresh: u8,
    /// Low LMT overload threshold in mV peak.
    pub lmt_overload_low_thresh_mv_peak: u16,
    /// High LMT overload threshold in mV peak.
    pub lmt_overload_high_thresh_mv_peak: u16,
}

/// Auto Gain Control (AGC) configuration.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GainControlAutoConfig {
    /// Prevent a gain increase on a small LMT or ADC overload.
    pub prevent_gain_inc_lmt_adc_small_overload: bool,
    /// Change the gain immediately on a large ADC overload.
    pub immed_gain_change_if_large_adc_overload: bool,
    /// Change the gain immediately on a large LMT overload.
    pub immed_gain_change_if_large_lmt_overload: bool,

    /// Decrement step size for a large LPF gain change or full table case 1.
    pub decrement_step_size_large_lpf_or_full_table_case_1: u4,
    /// Decrement step size for a large LMT overload or full table case 3.
    pub decrement_step_size_large_lmt_overload_full_table_case_3: u3,

    /// Exceed counter of the small ADC overload. Range 0 to 15.
    pub adc_small_overload_exceed_counter: u4,
    /// Exceed counter of the large ADC overload. Range 0 to 15.
    pub adc_large_overload_exceed_counter: u4,

    /// Exceed counter of the digital saturation. Range 0 to 15.
    pub dig_saturation_exceed_counter: u4,

    /// Exceed counter of the large LMT overload. Range 0 to 15.
    pub lmt_overload_large_exceed_counter: u4,
    /// Exceed counter of the small LMT overload. Range 0 to 15.
    pub lmt_overload_small_exceed_counter: u4,

    /// Offset to inner high threshold in dB/LSB.
    pub outer_thresh_offset_to_inner_high: u4,
    /// Inner high threshold in minus dBFS.
    pub inner_thresh_high_negative_dbfs: u7,
    /// Inner high threshold in minus dBFS. Must be a larger value than the inner high threshold
    /// so the threshold is lower.
    pub inner_thresh_low_negative_dbfs: u7,
    /// Offset to inner low threshold in dB/LSB.
    pub outer_thresh_offset_to_inner_low: u4,

    /// Equivalent to Gain A step decrease in figure 24 of the datasheet.
    pub outer_thresh_high_dec_steps: u4,
    /// Equivalent to Gain B step decrease in figure 24 of the datasheet.
    pub inner_thresh_high_dec_steps: u3,

    /// Equivalent to Gain C step increase in figure 24 of the datasheet.
    pub inner_thresh_low_inc_steps: u3,
    /// Equivalent to Gain D step increase in figure 24 of the datasheet.
    pub outer_thresh_low_inc_steps: u4,

    /// Extra margin of the attack delay in us. Range 0 to 31.
    pub attack_delay_extra_margin_us: u32,
    /// Allows resetting the gain counter with CTRL_IN2.
    pub enable_sync_for_gain_counter: bool,
}

/// Gain control input mode of the manual gain control with a split gain table.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SplitTableManualMode {
    /// The AGC determines the gain.
    Agc,
    /// Only the LPF gain changes.
    OnlyInLpf,
    /// Only the LMT gain changes.
    OnlyInLmt,
}

/// Manual gain control configuration.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct GainControlManualConfig {
    /// Must be in range 1..=8.
    pub dec_gain_step: u8,
    /// Must be in range 1..=8.
    pub inc_gain_step: u8,
    /// Use the pin control of RX1 instead of the SPI control.
    pub rx1_ctrl_input: bool,
    /// Use the pin control of RX2 instead of the SPI control.
    pub rx2_ctrl_input: bool,
    /// Gain control input mode for a split gain table. [`None`] if no split table is used.
    pub split_table_mode: Option<SplitTableManualMode>,
}

/// Fast AGC configuration.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GainControlFastAutoConfig {
    /* Fast AGC */
    /// State wait time in ns.
    pub state_wait_time_ns: u32,

    /* Fast AGC - Low Power */
    /// Low power: allows the AGC to increase the gain.
    pub allow_agc_gain_increase: bool,
    /// Low power threshold increment time in RX samples.
    pub lp_thresh_increment_time: u8,
    /// Low power threshold increment steps. Range 1 to 8.
    pub lp_thresh_increment_steps: u8,

    /* Fast AGC - Lock Level */
    /// Lock level: allows a gain increase of the LMT.
    pub lock_level_lmt_gain_increase: bool,
    /// Upper limit of the gain increase at the lock level. Range 0 to 63.
    pub lock_level_gain_increase_upper_limit: u6,

    /* Fast AGC - Peak Detectors and Final Settling */
    /// Final settling steps of the LPF after the lock level. Range 0 to 3.
    pub lpf_final_settling_steps: u2,
    /// Final settling steps of the LMT after the lock level. Range 0 to 3.
    pub lmt_final_settling_steps: u2,
    /// Final overrange count. Range 0 to 7.
    pub final_overrange_count: u3,

    /* Fast AGC - Final Power Test */
    /// Final power test: allows a gain increase after the gain lock.
    pub gain_increase_after_gain_lock: bool,

    /* Fast AGC - Unlocking the Gain */
    /// Use the last lock level for the set gain.
    pub use_last_lock_level_for_set_gain: bool,
    //pub rst_gla_stronger_sig_thresh_exceeded: bool,
    /// Offset of the optimized gain in steps. Range 0 to 15.
    pub optimized_gain_offset: u4,
    /// Stronger signal threshold above the lock level in dBFS, which resets the gain lock. Range 0
    /// to 63.
    pub rst_gla_stronger_sig_thresh_above_ll: u6,
    /// Energy lost signal threshold below the lock level, which resets the gain lock.
    pub rst_gla_energy_lost_sig_thresh_below_ll: u6,
    /// Gain lock exit count for the energy lost and stronger signal cases in RX samples. Range 0 to
    /// 63.
    pub energy_lost_stronger_sig_gain_lock_exit_cnt: u6,

    /// Suppress the gain unlock if the energy lost threshold is exceeded.
    pub suppress_gain_unlock_on_energy_lost_threshold_exceeded: bool,
    /// If the LMT overload suppression is enabled, the ADC overload will always be suppressed
    /// as well.
    pub suppress_gain_unlock_on_large_adc_large_lmt_overload: bool,
    /// Suppress the gain unlock on an ADC overload.
    pub suppress_gain_unlock_on_adc_overload: bool,
    /// Suppress the gain unlock if the stronger signal threshold is exceeded.
    pub suppress_gain_unlock_on_stronger_signal_threshold_exceeded: bool,

    /// Gain index the fast AGC goes to after the exit from the RX mode.
    pub gain_index_type_after_exit_rx_mode: FastAgcTargetGainIndexType,
    /// If this is None, the EN_AGC pin is not used.
    pub gain_index_type_on_en_agc_high: Option<FastAgcTargetGainIndexType>,
    /// Power measurement duration in state 5 in RX samples. Range 0 to 524288.
    pub power_measurement_duration_in_state5: u32,

    /// Decrement Step size for: Small LPF Gain Change or Full Table Case #2.
    pub decrement_step_small_lpf_gain_full_table_case_2: u3,
}

/// Received Signal Strength Indicator (RSSI) configuration.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RssiConfig {
    /// Event which restarts the RSSI measurement.
    pub restart_mode: RssiRestartMode,
    /// Delay, duration and wait time of the RSSI measurement.
    pub inner: TemporalConfig,
}

impl Default for RssiConfig {
    fn default() -> Self {
        Self {
            restart_mode: RssiRestartMode::EnAgcPinIsPulledHigh,
            inner: TemporalConfig::RxSamples {
                delay: 0,
                duration: 0,
                wait: 0,
            },
        }
    }
}

/// Unit and values of the RSSI timing.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TemporalConfig {
    /// Values in RX samples.
    RxSamples {
        /// Delay before the first measurement after a restart.
        delay: u32,
        /// Duration of a measurement.
        duration: u32,
        /// Wait time between measurements.
        wait: u32,
    },
    /// Values in us.
    TimeUs {
        /// Delay before the first measurement after a restart.
        delay_us: u32,
        /// Duration of a measurement.
        duration_us: u32,
        /// Wait time between measurements.
        wait_us: u32,
    },
}

/// Decimation of the auxiliary ADC as a register value.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AuxAdcDecimationBits(pub u3);

impl AuxAdcDecimationBits {
    /// Creates the decimation from the register value.
    pub const fn new(value: u3) -> Self {
        Self(value)
    }

    /// Actual decimation value, calculate per formula 256 * 2 to the power of the register value.
    pub const fn decimation(&self) -> u32 {
        256 * (1 << self.0.value() as u32)
    }
}

/// Auxiliary ADC configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct AuxAdcConfig {
    /// Decimation of the auxiliary ADC.
    pub decimation: AuxAdcDecimationBits,
    /// Clock rate of the auxiliary ADC in Hz.
    pub clock_rate_hz: NonZero<u32>,
    /// Temperature sensor configuration.
    pub temp_sense: TempSenseConfig,
}

/// Temperature sensor configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct TempSenseConfig {
    /// Decimation of the temperature sensor.
    pub decimation: AuxAdcDecimationBits,
    /// Interval between temperature measurements in ms.
    pub measurement_interval_ms: u16,
    /// Offset of the temperature sensor.
    pub offset_signed: i8,
    /// Enables periodic temperature measurements.
    pub enable_periodic: bool,
}
