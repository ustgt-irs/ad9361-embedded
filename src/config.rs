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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StepSizeKind {
    ManualGainInc,
    ManualGainDec,
    DigitalGain,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid RF RX bandwidth")]
    InvalidRfRxBandwidth,
    #[error("Invalid RF TX bandwidth")]
    InvalidRfTxBandwidth,
    #[error("Invalid CMOS data clock larger than 61.44 MSPS")]
    InvalidCmosDataClock,
    #[error("Invalid RX clock rate")]
    InvalidRxClock(RxClock),
    #[error("Invalid TX clock rate")]
    InvalidTxClock(TxClock),
    #[error("Invalid ADC overrange sample, must be in range 1..=8")]
    AdcOverrangeSampleInvalid(u8),
    #[error("Invalid step size for {0:?}, value: {1}")]
    InvalidStepSize(StepSizeKind, u8),
    #[error("invalid ADC thresholds")]
    InvalidAdcOverloadThresholds,
    #[error("invalid LMT overload thresholds")]
    InvalidLmtOverloadThreshold,
    #[error("invalid AGC thresholds")]
    InvalidAgcThresholds,
    #[error("TX attenuation invalid, out of bounds")]
    TxAttenuationInvalid(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirChSelect {
    Ch1,
    Ch2,
    Both,
}

impl FirChSelect {
    pub const fn channel1(&self) -> bool {
        matches!(self, FirChSelect::Ch1 | FirChSelect::Both)
    }

    pub const fn channel2(&self) -> bool {
        matches!(self, FirChSelect::Ch2 | FirChSelect::Both)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxFirConfig<'coef> {
    pub ch_select: FirChSelect,
    /// -6dB gain.
    pub attentuate_6db: bool,
    /// Coefficient for each taps. Only a part of this might be used, depending on the number
    /// of taps.
    pub tx_coefs: &'coef [i16],
    pub num_taps: regs::tx_filter_conf::NumberOfTapsRaw,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid number of coefficients")]
pub struct InvalidNumberOfCoefficientsError;

impl TxFirConfig<'_> {
    pub const fn tx_coeffs_size_equal_to_num_of_taps(&self) -> bool {
        self.tx_coefs.len() == self.num_taps.number() as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RxFirConfig<'coef> {
    pub ch_select: FirChSelect,
    pub gain: regs::rx_filter_gain::FilterGain,
    /// Coefficient for each taps. Only a part of this might be used, depending on the number
    /// of taps.
    pub rx_coefs: &'coef [i16],
    pub num_taps: regs::tx_filter_conf::NumberOfTapsRaw,
}

impl RxFirConfig<'_> {
    pub const fn rx_coeffs_size_equal_to_num_of_taps(&self) -> bool {
        self.rx_coefs.len() == self.num_taps.number() as usize
    }
}

/// Select between enabling all channels and only enabling one TX and one RX path.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TransceiverChannelMode {
    Dual,
    Single { rx: ReceiverId, tx: TransmitterId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelPortModeConfig(pub regs::parallel_port_conf_3::Register);

impl ParallelPortModeConfig {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExternalClockConfig {
    /// External oscillator connected to the REF_CLK_IN pin.
    Oscillator,
    /// External crystal in combination with internal digital programmable capacitor.
    Dcxo { coarse_tune: u6, fine_tune: u13 },
}

impl From<ExternalClockConfig> for regs::ExternalClockConfig {
    fn from(value: ExternalClockConfig) -> Self {
        match value {
            ExternalClockConfig::Oscillator => Self::Oscillator,
            ExternalClockConfig::Dcxo { .. } => Self::Dcxo,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TxMonitorConfig {
    Enabled(TxMonConfigParams),
    Disabled,
}

#[derive(Debug, Clone)]
pub struct ConfigRaw {
    // Clock configuration.
    pub reference_clk_rate: u32,

    /// RX bandwidth.
    pub rf_rx_bandwidth_hz: u32,
    /// TX bandwidth.
    pub rf_tx_bandwidth_hz: u32,

    /// Clock configuration.
    ///
    /// If `clock.rx.rx_fir`/`clock.tx.tx_fir` are set to a non-bypass value, `setup()` alone is
    /// not sufficient for correct operation. See the doc comments on those fields and on
    /// [`crate::Ad9361Uninit::setup`] for why a follow-up call to load real filter taps is
    /// required.
    pub clock: clocks::ClockConfig,
    /// Transceiver channel mode.
    pub transceiver_channel_mode: TransceiverChannelMode,
    /// Hardware configuration of the RX RF path.
    pub rx_path: RxRfPathConfig,
    /// Hardware configuration of the TX RF path.
    pub tx_path: TxRfPathConfig,

    pub tx_monitoring: TxMonitorConfig,

    /// Division duplex configuration, FDD or TDD.
    pub division_duplex_config: DivisionDuplexConfig,

    /// Digital Interface Control
    pub digital_interface: DigitalInterfaceConfig,

    pub external_clock_config: ExternalClockConfig,

    pub gain_table_type: RxGainTableType,

    /// If [None], the CLKOUT pin is driven to 0.
    pub clkout_mode: Option<ClkOutMode>,

    // ENSM config
    pub ensm_enable_pin_pulse_mode: bool,
    pub ensm_enable_txnrx_control: bool,

    pub tx_quad_calib: Option<crate::types::RxPhaseConfig>,

    pub dc_offset_tracking_update_event: u3,
    pub dc_offset_attenuation_high_range: u8,
    pub dc_offset_attenuation_low_range: u8,
    pub dc_offset_count_high_range: u8,
    pub dc_offset_count_low_range: u8,
    pub qec_tracking_slow_mode: bool,

    // TX Attenuation Control
    pub tx_attenuation_md_b: u32,
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
    pub ctrl_outs_enable_mask: u8,
    pub ctrl_outs_index: u8,

    /// External LNA Control
    pub elna: Option<ExternalLnaConfig>,

    pub gpo_config: GpoConfig,
}

impl ConfigRaw {
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
                    manual_config.inc_gain_step,
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

pub struct ConfigValidated(pub ConfigRaw);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DivisionDuplexMode {
    /// Frequency division duplex mode. In this mode, RX and TX can work at the same time on
    /// different frequency bands.
    Fdd,
    /// Time division duplex mode. In this mode, the RX and TX synthesizers are shared, and the
    /// device rapidly switches between RX and TX modes.
    Tdd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DivisionDuplexConfig {
    Fdd {
        independent_mode: bool,
    },
    Tdd {
        enable_txmon: bool,
        use_dual_synth: bool,
    },
}

impl DivisionDuplexConfig {
    pub const fn mode(&self) -> DivisionDuplexMode {
        match self {
            DivisionDuplexConfig::Fdd { .. } => DivisionDuplexMode::Fdd,
            DivisionDuplexConfig::Tdd { .. } => DivisionDuplexMode::Tdd,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuxDacConfig {
    pub aux_dac_manual_mode_enable: bool,
    pub dac_config: [Option<SingleDacConfig>; 2],
}

#[derive(Debug, Clone)]
pub struct TxMonConfigParams {
    /// Enable TX1 monitor.
    pub enable_tx1: bool,
    /// Enable TX2 monitor.
    pub enable_tx2: bool,

    pub low_high_gain_threshold_md_b: u32,
    pub low_gain_d_b: u5,
    pub high_gain_d_b: u5,
    pub tx_mon_track: bool,
    pub one_shot_mode: bool,
    pub tx_mon_delay: u10,
    pub tx_mon_duration: u32,
    pub tx1_mon_front_end_gain: u2,
    pub tx2_mon_front_end_gain: u2,
    pub tx1_mon_lo_cm: u6,
    pub tx2_mon_lo_cm: u6,
}

#[derive(Debug, Clone)]
pub struct SingleDacConfig {
    pub default_value_m_v: u32,

    pub active_in_rx: bool,
    pub active_in_tx: bool,
    pub active_in_alert: bool,

    pub rx_delay_us: u8,
    pub tx_delay_us: u8,
}

/// General Purpose Output (GPO) pin configuration.
#[derive(Debug, Clone)]
pub struct GpoConfig {
    /// When set, the GPOs can be controlled manually. Otherwise, the GPOs are ENSM slaves.
    pub manual_mode_enable: bool,
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
    pub init_state: bool,
    pub inactive_state_high: bool,
    pub slave_rx_enable: bool,
    pub slave_tx_enable: bool,
    pub rx_delay_us: u8,
    pub tx_delay_us: u8,
}

#[derive(Debug, Clone)]
pub struct DigitalInterfaceConfig {
    // PP1 register fields.
    pub pp1_config: Pp1Register,

    // PP2 register.
    pub pp2_config: Pp2Register,

    // PP3 register (parallel port mode config).
    pub pp3_config: ParallelPortModeConfig,

    // RX/TX interface delay, written to `RX_CLOCK_DATA_DELAY`/`TX_CLOCK_DATA_DELAY` at setup.
    //
    // Without the `axi-tune` feature, this is the delay, permanently. With it, this is just the
    // starting point: [`crate::Ad9361::digital_tune`] may later calculate and apply a different
    // value.
    pub rx_default_delay: RxClockDelayConfig,
    pub tx_default_delay: TxClockDelayConfig,

    pub lvds_config: Option<LvdsConfig>,
    pub rx1rx2_phase_inversion: bool,
}

#[derive(Debug, Clone)]
pub struct LvdsConfig {
    pub bias_control: BiasControlRegister,
    pub invert1_control: LvdsInversionRegister1,
    pub invert2_control: LvdsInversionRegister2,
}

impl LvdsConfig {
    pub fn new(lvds_bias_mv: u32, rx_onchip_termination: bool) -> Self {
        Self {
            bias_control: BiasControlRegister::ZERO
                .with_lvds_bias(Self::lvds_bias_reg_value(lvds_bias_mv))
                .with_rx_on_chip_term(rx_onchip_termination),
            invert1_control: LvdsInversionRegister1::new_with_raw_value(0xFF),
            invert2_control: LvdsInversionRegister2::new_with_raw_value(0x0F),
        }
    }

    pub const fn lvds_bias_reg_value(milli_volts: u32) -> u3 {
        u3::new(((milli_volts.saturating_sub(75) / 75) & 0x7) as u8)
    }
}

impl DigitalInterfaceConfig {
    pub const fn pp1_config(&self) -> regs::parallel_port_conf_1::Register {
        self.pp1_config
    }

    pub const fn pp2_config(&self) -> regs::parallel_port_conf_2::Register {
        self.pp2_config
    }

    pub const fn pp3_config(&self) -> regs::parallel_port_conf_3::Register {
        self.pp3_config.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExternalLnaConfig {
    pub settling_delay_ns: u32,
    pub gain_md_b: u32,
    pub bypass_loss_md_b: u32,
    pub rx1_gpo0_control: bool,
    pub rx2_gpo1_control: bool,
    pub enable_for_all_indexes_in_gt: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum GainControlConfigError {
    #[error("manual config required when either channel is in manual mode")]
    ManualConfigMissing,
    #[error("auto config required when either channel uses an auto AGC mode")]
    AutoConfigMissing,
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
    pub common: GainControlCommon,
}

impl GainControl {
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

    #[inline]
    pub const fn rx1_mode(&self) -> GainControlMode {
        self.rx1_mode
    }

    #[inline]
    pub const fn rx2_mode(&self) -> GainControlMode {
        self.rx2_mode
    }

    #[inline]
    pub const fn manual_mut(&mut self) -> Option<&mut GainControlManualConfig> {
        self.manual.as_mut()
    }

    #[inline]
    pub const fn auto_mut(&mut self) -> Option<&mut GainControlAutoConfig> {
        self.auto.as_mut()
    }

    #[inline]
    pub const fn fast_mut(&mut self) -> Option<&mut GainControlFastAutoConfig> {
        self.fast.as_mut()
    }

    #[inline]
    pub const fn manual(&self) -> Option<&GainControlManualConfig> {
        self.manual.as_ref()
    }

    #[inline]
    pub const fn auto(&self) -> Option<&GainControlAutoConfig> {
        self.auto.as_ref()
    }

    #[inline]
    pub const fn fast(&self) -> Option<&GainControlFastAutoConfig> {
        self.fast.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct DigitalGainConfig {
    pub max_dig_gain: u5,
    /// Must be in range 1..=8.
    pub dig_gain_step_size: u8,
}

/// Raw register value wrapper.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct DecPowerMeasurementDuration(pub u4);

impl DecPowerMeasurementDuration {
    pub const fn new(value: u4) -> Self {
        Self(value)
    }

    /// Corresponding RX cycles.
    pub const fn rx_cycles(&self) -> u32 {
        16 << self.0.value() as u32
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GainControlCommon {
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
    pub dec_pwr_meas_source: DecPowerMeasurementSource,
    // Not sure if this belongs here, but we try to remain close to the C lib for now.
    pub gain_update_interval_us: u32,
    // These can also be used in manual gain control mode.
    /// Look at the datasheet p.36 for more information how to calculate those.
    pub adc_small_overload_thresh: u8,
    /// Look at the datasheet p.36 for more information how to calculate those.
    pub adc_large_overload_thresh: u8,
    pub lmt_overload_low_thresh_mv_peak: u16,
    pub lmt_overload_high_thresh_mv_peak: u16,
}

/// Auto Gain Control (AGC) configuration.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GainControlAutoConfig {
    pub prevent_gain_inc_lmt_adc_small_overload: bool,
    pub immed_gain_change_if_large_adc_overload: bool,
    pub immed_gain_change_if_large_lmt_overload: bool,

    pub decrement_step_size_large_lpf_or_full_table_case_1: u4,
    pub decrement_step_size_large_lmt_overload_full_table_case_3: u3,

    pub adc_small_overload_exceed_counter: u4,
    pub adc_large_overload_exceed_counter: u4,

    pub dig_saturation_exceed_counter: u4,

    pub lmt_overload_large_exceed_counter: u4,
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

    pub attack_delay_extra_margin_us: u32,
    /// Allows resetting the gain counter with CTRL_IN2.
    pub enable_sync_for_gain_counter: bool,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SplitTableManualMode {
    Agc,
    OnlyInLpf,
    OnlyInLmt,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct GainControlManualConfig {
    /// Must be in range 1..=8.
    pub dec_gain_step: u8,
    /// Must be in range 1..=8.
    pub inc_gain_step: u8,
    pub rx1_ctrl_input: bool,
    pub rx2_ctrl_input: bool,
    pub split_table_mode: Option<SplitTableManualMode>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GainControlFastAutoConfig {
    /* Fast AGC */
    pub state_wait_time_ns: u32,

    /* Fast AGC - Low Power */
    pub allow_agc_gain_increase: bool,
    pub lp_thresh_increment_time: u8,
    pub lp_thresh_increment_steps: u8,

    /* Fast AGC - Lock Level */
    pub lock_level_lmt_gain_increase: bool,
    pub lock_level_gain_increase_upper_limit: u6,

    /* Fast AGC - Peak Detectors and Final Settling */
    pub lpf_final_settling_steps: u2,
    pub lmt_final_settling_steps: u2,
    pub final_overrange_count: u3,

    /* Fast AGC - Final Power Test */
    pub gain_increase_after_gain_lock: bool,

    /* Fast AGC - Unlocking the Gain */
    pub use_last_lock_level_for_set_gain: bool,
    //pub rst_gla_stronger_sig_thresh_exceeded: bool,
    pub optimized_gain_offset: u4,
    pub rst_gla_stronger_sig_thresh_above_ll: u6,
    pub rst_gla_energy_lost_sig_thresh_below_ll: u6,
    pub energy_lost_stronger_sig_gain_lock_exit_cnt: u6,

    pub suppress_gain_unlock_on_energy_lost_threshold_exceeded: bool,
    /// If the LMT overload suppression is enabled, the ADC overload will always be suppressed
    /// as well.
    pub suppress_gain_unlock_on_large_adc_large_lmt_overload: bool,
    pub suppress_gain_unlock_on_adc_overload: bool,
    pub suppress_gain_unlock_on_stronger_signal_threshold_exceeded: bool,

    pub gain_index_type_after_exit_rx_mode: FastAgcTargetGainIndexType,
    /// If this is None, the EN_AGC pin is not used.
    pub gain_index_type_on_en_agc_high: Option<FastAgcTargetGainIndexType>,
    pub power_measurement_duration_in_state5: u32,

    /// Decrement Step size for: Small LPF Gain Change or Full Table Case #2.
    pub decrement_step_small_lpf_gain_full_table_case_2: u3,
}

/// Received Signal Strength Indicator (RSSI) configuration.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RssiConfig {
    pub restart_mode: RssiRestartMode,
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TemporalConfig {
    RxSamples {
        delay: u32,
        duration: u32,
        wait: u32,
    },
    TimeUs {
        delay_us: u32,
        duration_us: u32,
        wait_us: u32,
    },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AuxAdcDecimationBits(pub u3);

impl AuxAdcDecimationBits {
    pub const fn new(value: u3) -> Self {
        Self(value)
    }

    /// Actual decimation value, calculate per formula 256 * 2 to the power of the register value.
    pub const fn decimation(&self) -> u32 {
        256 * (1 << self.0.value() as u32)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuxAdcConfig {
    pub decimation: AuxAdcDecimationBits,
    pub clock_rate_hz: NonZero<u32>,
    pub temp_sense: TempSenseConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TempSenseConfig {
    pub decimation: AuxAdcDecimationBits,
    pub measurement_interval_ms: u16,
    pub offset_signed: i8,
    pub enable_periodic: bool,
}
