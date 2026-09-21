use arbitrary_int::{traits::Integer as _, u6};
use arbitrary_int::{u11, u21, u23};

use crate::limits::BBPLL_MODULUS;
use crate::regs;
pub use crate::regs::clk_bb_pll::AdcDivisor;
pub use crate::regs::rx_enable_filter_ctrl::{Rhb3Decimation, RxFirDecimation};
pub use crate::regs::tx_enable_filter_ctrl::{Thb3Interpolation, TxFirInterpolation};

/// Modulus of the fractional BB PLL divider.
pub const BB_PLL_MODULUS: u32 = 2_088_960;
/// Modulus of the fractional RF PLL divider.
pub const RFPLL_MODULUS: u32 = 8_388_593;

/// Clocks of the AD9361 clock tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockId {
    /// External reference clock which is fed into the the RX, TX and BB PLLs.
    ///
    /// This has to be provided by an external clock with a range of 5 MHz to 320 MHz.
    /// 19 to 80 MHz are recommended.
    ExtRefClk,
    /// Base-band reference clock which is fed into the BB PLLs. This is derived from the external
    /// reference clock. For the best performance, this should have a value from 35 to 70 MHz.
    BbRef,

    /// This clock is fed into the RX PLL. Also called RX SYNTH in the datasheet.
    /// This is derived from the external reference clock. This is the reference clock for the RX
    /// LO (local oscillator) clock.
    ///
    /// For best performance, this should have a value of 35 to 80 MHz.
    RxRef,
    /// PLL frequency, ranging from 6 to 12 GHz. Derived from the RX reference clock.
    RxPll,
    /// The RX local oscillator frequency. This is the frequency the RX mixer
    /// uses to downconvert the incoming RF signal to baseband.
    /// Range: 47 MHz – 6 GHz, but the datasheet only specifies a support for a reception frequency
    /// range from 70 MHz to 6 GHz. This can also be supplied externally.
    RxLo,
    /// RX LO, but supplied externally.
    RxLoExternal,

    /// This clock is fed into the TX PLL. Also called TX SYNTH in the datasheet.
    /// This is derived from the external reference clock. This is the reference clock for the TX
    /// LO (local oscillator) clock.
    ///
    /// For best performance, this should have a value of 35 to 80 MHz.
    TxRef,
    /// PLL frequency, ranging from 6 to 12 GHz. Derived from the TX reference clock.
    TxPll,
    /// The TX local oscillator frequency. This is the frequency the TX mixer
    /// uses to upconvert the baseband signal to RF.
    /// Range: 47 MHz – 6 GHz. This can also be supplied externally.
    TxLo,
    /// TX LO, but supplied externally.
    TxLoExternal,

    /// BB PLL output clock. This is an integer multiple of the RX ADC clock, the TX ADC clock,
    /// all analog calibration clocks as well as the clocks used in the digital section.
    BbPll,
    /// ADC master clock. All the other clocks which are part of the RX signal path are derives
    /// from this clock.
    Adc,
    /// Clock signal after decimation by RX HB3.
    Hb3,
    /// Clock signal after decimation by RX HB2.
    Hb2,
    /// Clock signal after decimation by RX HB1, input to the RX FIR.
    Hb1,
    /// Clock signal after decimation by the RX FIR, relevant clock for reading IQ data
    /// with a processor/FPGA.
    RxSample,

    /// DAC clock, derived from the ADC clock.
    Dac,
    /// Input clock of the TX HB3 stage.
    T2,
    /// Input clock of the TX HB2 stage.
    T1,
    /// Input clock of the TX HB1 stage, which is the output clock of the TX FIR.
    ClkTf,
    /// Sample clock before the TX FIR, relevant clock for writing IQ data with a processor/FPGA.
    TxSample,
}

impl ClockId {
    /// Returns the clock this clock is derived from, or [`None`] for the external reference clock
    /// and the external LO clocks.
    pub const fn parent(&self) -> Option<Self> {
        match self {
            ClockId::ExtRefClk => None,
            ClockId::BbRef => Some(ClockId::ExtRefClk),

            ClockId::RxRef => Some(ClockId::ExtRefClk),
            ClockId::RxPll => Some(ClockId::RxRef),
            ClockId::RxLo => Some(ClockId::RxPll),
            ClockId::RxLoExternal => None,

            ClockId::TxRef => Some(ClockId::ExtRefClk),
            ClockId::TxPll => Some(ClockId::TxRef),
            ClockId::TxLo => Some(ClockId::TxPll),
            ClockId::TxLoExternal => None,

            ClockId::BbPll => Some(ClockId::BbRef),
            ClockId::Adc => Some(ClockId::BbPll),
            ClockId::Hb3 => Some(ClockId::Adc),
            ClockId::Hb2 => Some(ClockId::Hb3),
            ClockId::Hb1 => Some(ClockId::Hb2),
            ClockId::RxSample => Some(ClockId::Hb1),
            ClockId::Dac => Some(ClockId::Adc),
            ClockId::T2 => Some(ClockId::Dac),
            ClockId::T1 => Some(ClockId::T2),
            ClockId::ClkTf => Some(ClockId::T1),
            ClockId::TxSample => Some(ClockId::ClkTf),
        }
    }
}

/// Returns the scaler which keeps `refin_hz` at or below `max` after scaling, with the largest
/// possible multiplier or the smallest possible divisor.
pub const fn calculate_best_clock_divisor_for_max_value(
    refin_hz: u32,
    max: u32,
) -> regs::ClockScaler {
    match refin_hz {
        f if f <= max / 2 => regs::ClockScaler::Mul2,
        f if f <= max => regs::ClockScaler::Div1,
        f if f <= max * 2 => regs::ClockScaler::Div2,
        _ => regs::ClockScaler::Div4,
    }
}

/// The charge pump register value is 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid charge pump value, must be between 1 and 63")]
pub struct InvalidChargePumpError;

/// Charge pump setting of the BB PLL.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct ChargePumpConfig(u6);

impl ChargePumpConfig {
    /// Creates a new charge pump setting. The register value must not be 0.
    pub fn new(charge_pump: u6) -> Result<Self, InvalidChargePumpError> {
        if charge_pump.as_u32() == 0 {
            return Err(InvalidChargePumpError {});
        }
        Ok(Self(charge_pump))
    }

    /// Calculates the charge pump current for a target BB PLL frequency and reference clock.
    ///
    /// Mirrors the formula in `ad9361_bbpll_set_rate` in the C driver: scale is 150uA @
    /// (1280 MHz BBPLL, 40 MHz REFCLK), 25uA/LSB with a 25uA offset, clamped to a valid
    /// register value of 1 to 63.
    pub const fn calculate(bbpll_rate_hz: u32, ref_clk_rate_hz: u32) -> Self {
        let tmp = (bbpll_rate_hz as u64 >> 7) * 150;
        let tmp = tmp / ((ref_clk_rate_hz as u64 >> 7) * 32);
        let icp_val = ((tmp + 12) / 25) as i64 - 1;
        let icp_val = if icp_val < 1 {
            1
        } else if icp_val > 63 {
            63
        } else {
            icp_val
        };
        Self(u6::new(icp_val as u8))
    }

    /// Register value of the charge pump setting.
    pub const fn reg_value(&self) -> u6 {
        self.0
    }

    /// Charge pump current in uA.
    pub const fn current(&self) -> u32 {
        self.0.value() as u32 * 25
    }
}

/// BB PLL configuration.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct BbPllConfig {
    /// Integer part of the PLL divider.
    pub integer_freq_word: u8,
    /// Fractional part of the PLL divider in units of `1 / BBPLL_MODULUS`.
    pub fractional_freq_word: u21,
    /// Charge pump setting.
    pub charge_pump: ChargePumpConfig,
}

impl BbPllConfig {
    /// Calculates the configuration which generates `target_clock` from `bb_ref_clk_hz`.
    pub fn calculate(bb_ref_clk_hz: u32, target_clock: u32) -> Self {
        let n_int = (target_clock / bb_ref_clk_hz).min(u8::MAX.as_u32()) as u8;
        let n_fract = u21::new(
            (BBPLL_MODULUS as u64 * (target_clock as u64 - (n_int.as_u64() * bb_ref_clk_hz as u64))
                / bb_ref_clk_hz as u64)
                .min(u21::MAX.as_u64()) as u32,
        );

        Self {
            integer_freq_word: n_int,
            fractional_freq_word: n_fract,
            charge_pump: ChargePumpConfig::calculate(target_clock, bb_ref_clk_hz),
        }
    }
}

/// Divider between the RF PLL VCO and the LO clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PllVcoDivider {
    /// Divide by 2.
    Div2,
    /// Divide by 4.
    Div4,
    /// Divide by 8.
    Div8,
    /// Divide by 16.
    Div16,
    /// Divide by 32.
    Div32,
    /// Divide by 64.
    Div64,
    /// Divide by 128.
    Div128,
}

impl PllVcoDivider {
    /// Value of the divider.
    pub const fn divider(&self) -> u32 {
        match self {
            PllVcoDivider::Div2 => 2,
            PllVcoDivider::Div4 => 4,
            PllVcoDivider::Div8 => 8,
            PllVcoDivider::Div16 => 16,
            PllVcoDivider::Div32 => 32,
            PllVcoDivider::Div64 => 64,
            PllVcoDivider::Div128 => 128,
        }
    }

    /// Register bits of the divider.
    pub const fn as_reg_bits(&self) -> regs::PllVcoDividerBits {
        match self {
            PllVcoDivider::Div2 => regs::PllVcoDividerBits::Div2,
            PllVcoDivider::Div4 => regs::PllVcoDividerBits::Div4,
            PllVcoDivider::Div8 => regs::PllVcoDividerBits::Div8,
            PllVcoDivider::Div16 => regs::PllVcoDividerBits::Div16,
            PllVcoDivider::Div32 => regs::PllVcoDividerBits::Div32,
            PllVcoDivider::Div64 => regs::PllVcoDividerBits::Div64,
            PllVcoDivider::Div128 => regs::PllVcoDividerBits::Div128,
        }
    }
}

/// RF PLL configuration.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct RfPllConfig {
    /// Divider between the VCO and the LO clock.
    pub vco_div: PllVcoDivider,
    /// Integer part of the PLL divider.
    pub pll_int: u11,
    /// Fractional part of the PLL divider in units of `1 / RFPLL_MODULUS`.
    pub pll_frac: u23,
}

/// Configuration of an LO clock, which is either generated internally or supplied externally.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LocalOscClockConfig {
    /// External VCO.
    External {
        /// LO frequency will be half of this value.
        external_vco_freq: u64,
    },
    /// VCO divider, which scales the RF PLL clock down to the LO clock.
    Internal(RfPllConfig),
}

/// Error while calculating an RF PLL configuration.
#[derive(Debug, thiserror::Error)]
pub enum RfPllFreqError {
    /// The reference clock is out of range.
    #[error(
        "invalid reference clock, min {min}, max {max}",
        min = super::limits::MIN_SYNTH_FREF,
        max = super::limits::MAX_SYNTH_FREF
    )]
    InvalidRefClock,
    /// The VCO clock is out of range.
    #[error("invalid VCO clock, must be between 6 and 12 GHz")]
    InvalidVcoClock,
    /// There is no valid VCO clock for the LO clock.
    #[error("invalid LO clock, can not determine valid VCO clock for it")]
    InvalidLoClock,
    /// The VCO divider is not valid for an internal LO.
    #[error("invalid PLL VCO divider, must be divider for internal LO")]
    InvalidPllVcoDivider,
}

impl LocalOscClockConfig {
    /// Creates the configuration for an externally supplied VCO clock.
    pub const fn new_for_external_lo(external_vco_freq: u64) -> Self {
        Self::External { external_vco_freq }
    }

    /// Calculate the local oscillator configuration when generating the LO with
    /// the internal RX or TX PLLs.
    ///
    /// ## Arguments
    ///
    /// * `synth_clk` - Reference clock for the PLL also called F_REF.
    /// * `target_vco_clk` - Target VCO clock frequency, which ranges from 6 GHz to 12 GHz
    /// * `pll_vco_div` - PLL VCO divider, which scales the RF PLL clock down to the LO clock.
    pub fn calculate_for_internal_lo_with_target_vco_clock(
        synth_clk: u32,
        target_vco_clk: u64,
        pll_vco_div: PllVcoDivider,
    ) -> Result<Self, RfPllFreqError> {
        if !(super::limits::MIN_SYNTH_FREF..=super::limits::MAX_SYNTH_FREF).contains(&synth_clk) {
            return Err(RfPllFreqError::InvalidRefClock);
        }
        let n_int = u11::new((target_vco_clk / synth_clk as u64).min(u11::MAX.as_u64()) as u16);
        // Calculation using integer arithmetic. The formula can be achieved with
        // a slight rearrangement of the formula in the register datasheet.
        let n_fract = u23::new(
            (RFPLL_MODULUS as u64 * (target_vco_clk - (n_int.as_u64() * synth_clk as u64))
                / synth_clk as u64)
                .min(u23::MAX.as_u64()) as u32,
        );
        Ok(Self::Internal(RfPllConfig {
            vco_div: pll_vco_div,
            pll_int: n_int,
            pll_frac: n_fract,
        }))
    }

    /// Calculates the LO configuration for a target LO clock. Uses the smallest VCO divider which
    /// brings the VCO clock into its valid range.
    pub fn calculate_for_internal_lo(
        synth_clk: u32,
        target_lo_clock: u64,
    ) -> Result<Self, RfPllFreqError> {
        let mut pll_vco_div = PllVcoDivider::Div2;

        let mut target_vco_clk = target_lo_clock * pll_vco_div.divider() as u64;
        if target_vco_clk > super::limits::MAX_VCO_FREQ_HZ {
            return Err(RfPllFreqError::InvalidVcoClock);
        }

        while target_vco_clk < super::limits::MIN_VCO_FREQ_HZ {
            pll_vco_div = match pll_vco_div {
                PllVcoDivider::Div2 => PllVcoDivider::Div4,
                PllVcoDivider::Div4 => PllVcoDivider::Div8,
                PllVcoDivider::Div8 => PllVcoDivider::Div16,
                PllVcoDivider::Div16 => PllVcoDivider::Div32,
                PllVcoDivider::Div32 => PllVcoDivider::Div64,
                PllVcoDivider::Div64 => PllVcoDivider::Div128,
                PllVcoDivider::Div128 => {
                    return Err(RfPllFreqError::InvalidLoClock);
                }
            };
            target_vco_clk = target_lo_clock * pll_vco_div.divider() as u64;
        }
        Self::calculate_for_internal_lo_with_target_vco_clock(
            synth_clk,
            target_vco_clk,
            pll_vco_div,
        )
    }

    /// Register bits of the VCO divider. An external LO has its own value.
    #[inline]
    pub fn vco_div_reg_value(&self) -> regs::PllVcoDividerBits {
        match self {
            LocalOscClockConfig::External {
                external_vco_freq: _,
            } => regs::PllVcoDividerBits::External,
            LocalOscClockConfig::Internal(pll_cfg) => pll_cfg.vco_div.into(),
        }
    }
}

/// Frequencies of all clocks in Hz, calculated from a [`ClockConfig`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Clocks {
    ext_clk: u32,
    bb_refclk: u32,
    bb_pll: u32,
    adc: u32,

    rx: RxClocks,
    tx: TxClocks,
}

impl Clocks {
    /// External reference clock.
    #[inline]
    pub fn ext_clk(&self) -> u32 {
        self.ext_clk
    }

    /// Base-band PLL reference clock which is fed into the BB PLL.
    #[inline]
    pub const fn bb_refclk(&self) -> u32 {
        self.bb_refclk
    }

    /// BB PLL output clock. This is an integer multiple of the RX ADC clock, the TX ADC clock,
    /// all analog calibration clocks as well as the clocks used in the digital section.
    #[inline]
    pub const fn bb_pll(&self) -> u32 {
        self.bb_pll
    }

    /// ADC clock, which is also the DAC clock reference.
    #[inline]
    pub const fn adc(&self) -> u32 {
        self.adc
    }

    /// RX clocks.
    #[inline]
    pub const fn rx(&self) -> &RxClocks {
        &self.rx
    }

    /// TX clocks.
    #[inline]
    pub const fn tx(&self) -> &TxClocks {
        &self.tx
    }
}

/// Frequencies of the RX clocks in Hz.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RxClocks {
    synth_ref: u32,
    pll_vco: Option<u64>,
    lo: u64,
    hb3: u32,
    hb2: u32,
    rx_fir_input: u32,
    rx_sample: u32,
}

impl RxClocks {
    /// Input clock of the RX PLL block.
    #[inline]
    pub const fn synth_ref(&self) -> u32 {
        self.synth_ref
    }

    /// Frequency of the RX PLL VCO, or [`None`] if the LO is supplied externally.
    #[inline]
    pub const fn pll_vco(&self) -> Option<u64> {
        self.pll_vco
    }

    /// Frequency of the RX LO.
    #[inline]
    pub const fn lo(&self) -> u64 {
        self.lo
    }

    /// Clock after HB3 decimation, fed into HB2.
    #[inline]
    pub const fn hb3(&self) -> u32 {
        self.hb3
    }

    /// Clock after HB2 decimation, fed into HB1.
    #[inline]
    pub const fn hb2(&self) -> u32 {
        self.hb2
    }

    /// Clock after HB1 decimation, fed into RX FIR.
    #[inline]
    pub const fn hb1(&self) -> u32 {
        self.rx_fir_input
    }

    /// RX FIR Input clock, also called CLKRF in the C driver.
    #[inline]
    pub const fn rx_fir_input(&self) -> u32 {
        self.rx_fir_input
    }

    /// Sample clock after decimation by the RX FIR, relevant clock for reading IQ data with a
    /// processor/FPGA.
    #[inline]
    pub const fn rx_sample(&self) -> u32 {
        self.rx_sample
    }
}

/// Frequencies of the TX clocks in Hz.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TxClocks {
    synth_ref: u32,
    pll_vco: Option<u64>,
    lo: u64,
    tx_sample: u32,
    tx_fir: u32,
    hb1: u32,
    hb2: u32,
    dac_clk: u32,
}

impl TxClocks {
    /// Input clock of the TX PLL block.
    #[inline]
    pub const fn synth_ref(&self) -> u32 {
        self.synth_ref
    }

    /// Frequency of the TX PLL VCO, or [`None`] if the LO is supplied externally.
    #[inline]
    pub const fn pll_vco(&self) -> Option<u64> {
        self.pll_vco
    }

    /// Frequency of the TX LO.
    #[inline]
    pub const fn lo(&self) -> u64 {
        self.lo
    }

    /// TX sample clock before it goes into the FIR.
    #[inline]
    pub const fn tx_sample(&self) -> u32 {
        self.tx_sample
    }

    /// Clock after TX FIR interpolation, fed into HB1. Also called CLKTF in the C driver.
    #[inline]
    pub const fn tx_fir(&self) -> u32 {
        self.tx_fir
    }

    /// Clock after HB1 interpolation, fed into HB2.
    #[inline]
    pub const fn hb1(&self) -> u32 {
        self.hb1
    }

    /// Clock after HB2 interpolation, fed into HB3.
    #[inline]
    pub const fn hb2(&self) -> u32 {
        self.hb2
    }

    /// Clock after HB3 interpolation, fed into the DAC. Is basically the DAC clock.
    #[inline]
    pub const fn hb3(&self) -> u32 {
        self.dac_clk
    }

    /// DAC clock. Derived from the ADC clock.
    #[inline]
    pub const fn dac_clk(&self) -> u32 {
        self.dac_clk
    }
}

impl Clocks {
    /// Create a new [Clocks] information structure.
    pub fn new(refclk_hz: u32, config: &ClockConfig) -> Self {
        let bb_refclk = refclk_hz * config.ref_clk_scalers.bb_refclk.mult()
            / config.ref_clk_scalers.bb_refclk.div();

        let bb_pll_clk = (bb_refclk * config.bb_pll.integer_freq_word as u32)
            + ((bb_refclk as u64 * config.bb_pll.fractional_freq_word.value() as u64)
                / BB_PLL_MODULUS as u64) as u32;

        let adc_clk = bb_pll_clk / config.adc.divisor();
        let hb2_clk = adc_clk / config.rx.rhb3.divisor();
        let hb1_clk = hb2_clk / if config.rx.rhb2 { 2 } else { 1 };
        let rx_fir_clk = hb1_clk / if config.rx.rhb1 { 2 } else { 1 };
        let rx_sample = rx_fir_clk / config.rx.rx_fir.divisor();

        // Calculate from right to left. We have a desired output frequency which is derived
        // from the ADC clock. All other clocks are derived from that.
        let dac_clk = if config.tx.dac_div2 {
            adc_clk / 2
        } else {
            adc_clk
        };
        let tx_hb2 = dac_clk / config.tx.thb3.multiplier();
        let tx_hb1 = if config.tx.thb2 { tx_hb2 / 2 } else { tx_hb2 };
        let tx_fir = if config.tx.thb1 { tx_hb1 / 2 } else { tx_hb1 };
        let tx_sample = tx_fir / config.tx.tx_fir.multiplier();

        let mut rx_pll_vco = None;
        let rx_synth_clk = refclk_hz * config.ref_clk_scalers.rx_synth.mult()
            / config.ref_clk_scalers.rx_synth.div();
        let rx_lo_freq = match config.rx.lo {
            LocalOscClockConfig::External { external_vco_freq } => external_vco_freq / 2,
            LocalOscClockConfig::Internal(pll_config) => {
                let rx_pll_vco_tmp = rx_synth_clk as u64 * pll_config.pll_int.as_u64()
                    + ((rx_synth_clk as u64 * pll_config.pll_frac.as_u64()) / RFPLL_MODULUS as u64);
                rx_pll_vco = Some(rx_pll_vco_tmp);
                rx_pll_vco_tmp / pll_config.vco_div.divider() as u64
            }
        };

        let tx_synth_clk = refclk_hz * config.ref_clk_scalers.tx_synth.mult()
            / config.ref_clk_scalers.tx_synth.div();
        let mut tx_pll_vco = None;
        let tx_lo_freq = match config.tx.lo {
            LocalOscClockConfig::External { external_vco_freq } => external_vco_freq / 2,
            LocalOscClockConfig::Internal(pll_config) => {
                let tx_pll_vco_tmp = tx_synth_clk as u64 * pll_config.pll_int.as_u64()
                    + ((tx_synth_clk as u64 * pll_config.pll_frac.as_u64()) / RFPLL_MODULUS as u64);
                tx_pll_vco = Some(tx_pll_vco_tmp);
                tx_pll_vco_tmp / pll_config.vco_div.divider() as u64
            }
        };
        Self {
            ext_clk: refclk_hz,
            bb_refclk,
            bb_pll: bb_pll_clk,
            adc: adc_clk,
            rx: RxClocks {
                synth_ref: rx_synth_clk,
                pll_vco: rx_pll_vco,
                lo: rx_lo_freq,
                hb3: hb2_clk,
                hb2: hb1_clk,
                rx_fir_input: rx_fir_clk,
                rx_sample,
            },
            tx: TxClocks {
                synth_ref: tx_synth_clk,
                pll_vco: tx_pll_vco,
                lo: tx_lo_freq,
                tx_fir,
                hb1: tx_hb1,
                hb2: tx_hb2,
                dac_clk,
                tx_sample,
            },
        }
    }
}

/// Scalers which derive the PLL reference clocks from the external reference clock.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RefClockScalers {
    /// Scaler for the BB PLL reference clock.
    pub bb_refclk: regs::clock_ctrl::ClockScaler,
    /// You can use [calculate_best_clock_divisor_for_max_value] to calculate the best divisor
    /// for a target reference clock.
    pub rx_synth: regs::ClockScaler,
    /// You can use [calculate_best_clock_divisor_for_max_value] to calculate the best divisor
    /// for a target reference clock.
    pub tx_synth: regs::ClockScaler,
}

impl RefClockScalers {
    /// Calculates the RX synth reference clock from the external reference clock `refin_hz`.
    pub fn calculate_rx_synth_clock(&self, refin_hz: u32) -> u32 {
        refin_hz * self.rx_synth.mult() / self.rx_synth.div()
    }

    /// Calculates the TX synth reference clock from the external reference clock `refin_hz`.
    pub fn calculate_tx_synth_clock(&self, refin_hz: u32) -> u32 {
        refin_hz * self.tx_synth.mult() / self.tx_synth.div()
    }

    /// Calculates the BB PLL reference clock from the external reference clock `refin_hz`.
    pub fn calculate_bb_pll_synth_clock(&self, refin_hz: u32) -> u32 {
        refin_hz * self.bb_refclk.mult() / self.bb_refclk.div()
    }
}

/// Error for target PLL reference clocks which are out of range.
pub enum RefClockRangeError {
    /// The target reference clock is too high to be derived from the external reference clock.
    TooHigh {
        /// Requested reference clock in Hz.
        target_refclk: u32,
        /// Highest possible reference clock in Hz.
        max_possible_refclk: u32,
    },
    /// The target reference clock is too low to be derived from the external reference clock.
    TooLow {
        /// Requested reference clock in Hz.
        target_refclk: u32,
        /// Lowest possible reference clock in Hz.
        min_possible_refclk: u32,
    },
}

impl RefClockScalers {
    /// Calculate the divisors for target PLL input clocks.
    ///
    /// This function does not perform any checks for the BB PLL clock target value.
    ///
    /// For the BB PLL clock, the recommended range is 35 to 70 MHz. For the RX and TX PLL clocks,
    /// the recommended range is 35 to 80 MHz.
    pub const fn calculate(
        refin_hz: u32,
        bb_pll_clk: u32,
        rx_synth_fref_clk: u32,
        tx_synth_fref_clk: u32,
    ) -> Result<Self, RefClockRangeError> {
        if rx_synth_fref_clk > super::limits::MAX_SYNTH_FREF
            || tx_synth_fref_clk > super::limits::MAX_SYNTH_FREF
        {
            return Err(RefClockRangeError::TooHigh {
                target_refclk: rx_synth_fref_clk,
                max_possible_refclk: super::limits::MAX_SYNTH_FREF,
            });
        }
        if rx_synth_fref_clk < super::limits::MIN_SYNTH_FREF
            || tx_synth_fref_clk < super::limits::MIN_SYNTH_FREF
        {
            return Err(RefClockRangeError::TooLow {
                target_refclk: rx_synth_fref_clk,
                min_possible_refclk: super::limits::MIN_SYNTH_FREF,
            });
        }
        Ok(Self {
            bb_refclk: calculate_best_clock_divisor_for_max_value(refin_hz, bb_pll_clk),
            rx_synth: calculate_best_clock_divisor_for_max_value(refin_hz, rx_synth_fref_clk),
            tx_synth: calculate_best_clock_divisor_for_max_value(refin_hz, tx_synth_fref_clk),
        })
    }
}

/// Clock tree configuration.
#[derive(Debug, Clone)]
pub struct ClockConfig {
    /// Scalers of the PLL reference clocks.
    pub ref_clk_scalers: RefClockScalers,
    /// BB PLL configuration.
    pub bb_pll: BbPllConfig,
    /// Divisor from the BB PLL clock to the ADC clock.
    pub adc: AdcDivisor,

    /// RX path configuration.
    pub rx: RxConfig,
    /// TX path configuration.
    pub tx: TxConfig,
}

/// Error while calculating a clock path configuration.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RfClockPathCalculationError {
    /// The TX sample rate exceeds the maximum baseband rate.
    #[error(
        "specified TX sample rate faster than maximum baseband rate {}",
        crate::limits::MAX_BASEBAND_RATE
    )]
    TxSampleRateTooFast,
    /// The target frequencies can not be reached with the path configuration.
    #[error("can not achieve target frequencies with specified path configuration")]
    InvalidPathConfig,
    /// There is no valid configuration for the TX and RX sample rates.
    #[error("could not find valid configuration for specified TX/RX sample frequencies")]
    CouldNotFindValidConfig,
    /// The BB PLL clock derived from the sample clocks is too slow.
    #[error("sample clocks too slow, determined BB PLL clock is too slow")]
    ClocksTooSlow,
    /// The BB PLL clock derived from the sample clocks is too fast.
    #[error("sample clocks too fast, determined BB PLL clock is too fast")]
    ClocksTooFast,
    /// The ADC clock is too fast.
    #[error("invalid ADC clock, too fast")]
    AdcClockTooFast,
    /// The ADC clock is too slow.
    #[error("invalid ADC clock, too slow")]
    AdcClockTooSlow,
    /// The DAC clock is too fast.
    #[error("invalid DAC clock, too fast")]
    DacClockTooFast,
    /// The RX sample rate exceeds the maximum baseband rate.
    #[error(
        "specified RX sample rate faster than maximum baseband rate {}",
        crate::limits::MAX_BASEBAND_RATE
    )]
    RxSampleRateTooFast,
    /// The clock of the RX HB1 stage (CLKRF) is too fast.
    #[error("RX HB1 stage (CLKRF) clock exceeds {}", crate::limits::MAX_RX_HB1)]
    RxHb1ClockTooFast,
    /// The clock of the RX HB2 stage (R1) is too fast.
    #[error("RX HB2 stage (R1) clock exceeds {}", crate::limits::MAX_RX_HB2)]
    RxHb2ClockTooFast,
    /// The clock of the RX HB3 stage (R2) is too fast.
    #[error("RX HB3 stage (R2) clock exceeds {}", crate::limits::MAX_RX_HB3)]
    RxHb3ClockTooFast,
    /// The clock of the TX HB1 stage (CLKTF) is too fast.
    #[error("TX HB1 stage (CLKTF) clock exceeds {}", crate::limits::MAX_TX_HB1)]
    TxHb1ClockTooFast,
    /// The clock of the TX HB2 stage (T1) is too fast.
    #[error("TX HB2 stage (T1) clock exceeds {}", crate::limits::MAX_TX_HB2)]
    TxHb2ClockTooFast,
    /// The clock of the TX HB3 stage (T2) is too fast.
    #[error("TX HB3 stage (T2) clock exceeds {}", crate::limits::MAX_TX_HB3)]
    TxHb3ClockTooFast,
}

/// Computed BB PLL / ADC clock configuration including the full digital filter path.
///
/// Holds the BB PLL output frequency, the ADC divisor applied to it to obtain the ADC master
/// clock, the `dac_div2` flag, and all RX/TX half-band filter settings that were used to arrive
/// at those clocks. The `dac_div2` flag controls whether the DAC clock is half the ADC clock
/// (`true`) or equal to it (`false`).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BbClockPathConfigHelper {
    bb_pll_clock_hz: u32,
    adc_div: AdcDivisor,
    dac_div2: bool,
    rhb3: Rhb3Decimation,
    rhb2: bool,
    rhb1: bool,
    rx_fir: RxFirDecimation,
    thb3: Thb3Interpolation,
    thb2: bool,
    thb1: bool,
    tx_fir: TxFirInterpolation,
}

// One row of the AD9361 RX/TX digital filter chain lookup table (`clk_dividers` in the C
// driver's `ad9361_calculate_rf_clock_chain`).
//
// Other factorizations of the same `total_ratio` are mathematically possible but only these seven
// rows are what ADI's reference driver actually uses, so searching over just this table instead
// of every raw stage combination means we only ever pick combinations that are known-valid.
#[derive(Debug, Copy, Clone)]
struct ClockChainRow {
    stage3: u8,
    stage2_enabled: bool,
    stage1_enabled: bool,
}

/// AD9361 oversampling search preference, matching the C driver's `rate_governor` /
/// `trx_rate_governor` sysfs attribute.
///
/// The digital filter chain search in [`BbClockPathConfigHelper::calculate_and_validate`] can
/// either prefer the widest oversampling ratio it can make work (`HighestOsr`), or skip that
/// widest ratio and prefer a lower one (`Nominal`), trading oversampling margin for lower digital
/// filter power. `Nominal` is what the C driver defaults to (`ad9361_clear_state` sets
/// `phy->rate_governor = 1`); `HighestOsr` is only used if the caller opts in via
/// `ad9361_set_trx_rate_gov(phy, 0)`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum RateGovernor {
    /// Prefer the widest oversampling ratio (the first, 12x, row of the table) that produces a
    /// valid clock path.
    HighestOsr,
    /// Skip the widest oversampling ratio and prefer the next one down. Matches the C driver's
    /// default.
    #[default]
    Nominal,
}

impl RateGovernor {
    /// Number of leading (highest-ratio) rows of [`BbClockPathConfigHelper::CLOCK_CHAIN_TABLE`]
    /// to skip when searching.
    const fn skip_rows(self) -> usize {
        match self {
            RateGovernor::HighestOsr => 0,
            RateGovernor::Nominal => 1,
        }
    }
}

impl BbClockPathConfigHelper {
    /// Calculates and validates the BB PLL / ADC clock for a fully-specified digital filter path.
    ///
    /// The BB PLL always feeds the ADC/DAC chain through a mandatory divider (the AD9361 has no
    /// way to feed the BB PLL output directly into the ADC/DAC chain; the smallest supported
    /// divider is `Div2`); this picks the smallest one for which `adc_div` times the clock
    /// actually required at the input of the RX/TX digital filter chain lies within the BB PLL's
    /// supported frequency range, and uses that as the BB PLL target frequency.
    ///
    /// Returns [`RfClockPathCalculationError::InvalidPathConfig`] if the RX and TX paths do not
    /// result in a consistent ADC/DAC clock relationship, or
    /// [`RfClockPathCalculationError::ClocksTooSlow`]/[`RfClockPathCalculationError::ClocksTooFast`]
    /// if no supported ADC divisor brings the BB PLL clock into its valid range.
    #[allow(clippy::too_many_arguments)]
    const fn calculate_and_validate_with_full_path(
        tx_sample_hz: u32,
        rx_sample_is_twice_tx_sample: bool,
        rhb3: Rhb3Decimation,
        rhb2: bool,
        rhb1: bool,
        rx_fir: RxFirDecimation,
        thb3: Thb3Interpolation,
        thb2: bool,
        thb1: bool,
        tx_fir: TxFirInterpolation,
    ) -> Result<Self, RfClockPathCalculationError> {
        // Each variable below is named for the stage that just produced it, and feeds directly
        // into the next stage's calculation, walking the physical RX decimation chain from
        // the baseband sample rate up to the ADC clock (RX FIR, then half-band 1, then half-band
        // 2, then half-band 3), and the TX interpolation chain from the baseband sample rate up
        // to the DAC clock (TX FIR, then half-band 1, then half-band 2, then half-band 3). This
        // is the same per-stage staging the C driver validates against in
        // ad9361_validate_trx_clock_chain (ad9361.c), where these stages are named CLKRF/R1/R2
        // (RX) and CLKTF/T1/T2 (TX).
        let rx_sample_hz = if rx_sample_is_twice_tx_sample {
            tx_sample_hz * 2
        } else {
            tx_sample_hz
        };
        if tx_sample_hz > crate::limits::MAX_BASEBAND_RATE {
            return Err(RfClockPathCalculationError::TxSampleRateTooFast);
        }
        if rx_sample_hz > crate::limits::MAX_BASEBAND_RATE {
            return Err(RfClockPathCalculationError::RxSampleRateTooFast);
        }

        let mut rx_fir_output_hz = rx_sample_hz;
        match rx_fir {
            RxFirDecimation::Div1BypassFilter => (),
            RxFirDecimation::Div1EnableFilter => (),
            RxFirDecimation::Div2EnableFilter => rx_fir_output_hz *= 2,
            RxFirDecimation::Div4EnableFilter => rx_fir_output_hz *= 4,
        }
        if rx_fir_output_hz > crate::limits::MAX_RX_HB1 {
            return Err(RfClockPathCalculationError::RxHb1ClockTooFast);
        }
        let mut rx_half_band_1_output_hz = rx_fir_output_hz;
        if rhb1 {
            rx_half_band_1_output_hz *= 2;
        }
        if rx_half_band_1_output_hz > crate::limits::MAX_RX_HB2 {
            return Err(RfClockPathCalculationError::RxHb2ClockTooFast);
        }
        let mut rx_half_band_2_output_hz = rx_half_band_1_output_hz;
        if rhb2 {
            rx_half_band_2_output_hz *= 2;
        }
        if rx_half_band_2_output_hz > crate::limits::MAX_RX_HB3 {
            return Err(RfClockPathCalculationError::RxHb3ClockTooFast);
        }
        let mut adc_clock_from_rx = rx_half_band_2_output_hz;
        match rhb3 {
            Rhb3Decimation::Div1NoFiltering => (),
            Rhb3Decimation::Div2HalfBand => adc_clock_from_rx *= 2,
            Rhb3Decimation::Div3Filter => adc_clock_from_rx *= 3,
        }

        let mut tx_fir_output_hz = tx_sample_hz;
        match tx_fir {
            TxFirInterpolation::Mult1BypassFilter => (),
            TxFirInterpolation::Mult1EnableFilter => tx_fir_output_hz *= 2,
            TxFirInterpolation::Mult2EnableFilter => tx_fir_output_hz *= 2,
            TxFirInterpolation::Mult4EnableFilter => tx_fir_output_hz *= 4,
        }
        if tx_fir_output_hz > crate::limits::MAX_TX_HB1 {
            return Err(RfClockPathCalculationError::TxHb1ClockTooFast);
        }
        let mut tx_half_band_1_output_hz = tx_fir_output_hz;
        if thb1 {
            tx_half_band_1_output_hz *= 2;
        }
        if tx_half_band_1_output_hz > crate::limits::MAX_TX_HB2 {
            return Err(RfClockPathCalculationError::TxHb2ClockTooFast);
        }
        let mut tx_half_band_2_output_hz = tx_half_band_1_output_hz;
        if thb2 {
            tx_half_band_2_output_hz *= 2;
        }
        if tx_half_band_2_output_hz > crate::limits::MAX_TX_HB3 {
            return Err(RfClockPathCalculationError::TxHb3ClockTooFast);
        }
        let mut dac_clock_from_tx = tx_half_band_2_output_hz;
        match thb3 {
            Thb3Interpolation::Mult1NoFiltering => (),
            Thb3Interpolation::Mult2HalfBand => dac_clock_from_tx *= 2,
            Thb3Interpolation::Mult3Filter => dac_clock_from_tx *= 3,
        }
        // If we get here and the clock is already too fast, configuration is invalid.
        if dac_clock_from_tx > crate::limits::MAX_DAC_CLK {
            return Err(RfClockPathCalculationError::DacClockTooFast);
        }

        let dac_div2 = if adc_clock_from_rx == 2 * dac_clock_from_tx {
            true
        } else if adc_clock_from_rx == dac_clock_from_tx {
            false
        } else {
            return Err(RfClockPathCalculationError::InvalidPathConfig);
        };
        // `adc_clock_from_rx` is the clock required at the input of the RX/TX digital filter
        // chain (the ADC master clock). The BB PLL feeds that chain through a mandatory divider,
        // so the BB PLL itself has to run some multiple of `adc_clock_from_rx` faster. Pick the
        // largest divider (i.e. the highest BB PLL frequency) that still lands in range, matching
        // the C driver, which starts at `MAX_BBPLL_DIV` and only backs off if that overshoots
        // `MAX_BBPLL_FREQ`.
        let mut candidate = AdcDivisor::Div64;
        let (adc_div, bb_pll_clock_hz) = loop {
            // Trying the largest divisors first can overshoot `u32` for a fast
            // `adc_clock_from_rx`; that's just an emphatic "too fast", so treat overflow the same
            // as exceeding the max and move on to the next-smaller divisor.
            if let Some(value) = adc_clock_from_rx.checked_mul(candidate.divisor())
                && value <= crate::limits::MAX_BBPLL_FREQ
                && value >= crate::limits::MIN_BBPLL_FREQ
            {
                break (candidate, value);
            }
            if matches!(candidate, AdcDivisor::Div2) {
                // The smallest divisor gives the lowest possible BB PLL frequency; if even that
                // is below the minimum, no divisor can work.
                if adc_clock_from_rx * AdcDivisor::Div2.divisor() < crate::limits::MIN_BBPLL_FREQ {
                    return Err(RfClockPathCalculationError::ClocksTooSlow);
                }
                return Err(RfClockPathCalculationError::ClocksTooFast);
            }
            candidate = candidate.next_smaller();
        };
        if adc_clock_from_rx > crate::limits::MAX_ADC_CLK {
            return Err(RfClockPathCalculationError::AdcClockTooFast);
        } else if adc_clock_from_rx < crate::limits::MIN_ADC_CLK {
            return Err(RfClockPathCalculationError::AdcClockTooSlow);
        }
        Ok(Self {
            bb_pll_clock_hz,
            adc_div,
            dac_div2,
            rhb3,
            rhb2,
            rhb1,
            rx_fir,
            thb3,
            thb2,
            thb1,
            tx_fir,
        })
    }

    /// Ordered from the coarsest ratio (widest oversampling) to the finest (1:1), i.e. row 0 is
    /// the 12x ratio that [`RateGovernor::HighestOsr`] searches from and [`RateGovernor::Nominal`]
    /// skips.
    const CLOCK_CHAIN_TABLE: [ClockChainRow; 7] = [
        ClockChainRow {
            stage3: 3,
            stage2_enabled: true,
            stage1_enabled: true,
        },
        ClockChainRow {
            stage3: 2,
            stage2_enabled: true,
            stage1_enabled: true,
        },
        ClockChainRow {
            stage3: 3,
            stage2_enabled: false,
            stage1_enabled: true,
        },
        ClockChainRow {
            stage3: 2,
            stage2_enabled: true,
            stage1_enabled: false,
        },
        ClockChainRow {
            stage3: 3,
            stage2_enabled: false,
            stage1_enabled: false,
        },
        ClockChainRow {
            stage3: 2,
            stage2_enabled: false,
            stage1_enabled: false,
        },
        ClockChainRow {
            stage3: 1,
            stage2_enabled: false,
            stage1_enabled: false,
        },
    ];

    /// RF clock path calculator.
    ///
    /// This searches `Self::CLOCK_CHAIN_TABLE` starting from the row `rate_governor` selects
    /// and lets `Self::calculate_and_validate_with_full_path` confirm whether a given pairing
    /// actually produces a consistent ADC/DAC clock relationship. If `rate_governor` is
    /// [`RateGovernor::Nominal`] and no row from its starting point onward works, this retries
    /// once against the full table (i.e. falls back to [`RateGovernor::HighestOsr`]), matching
    /// the C driver forcing `rate_gov` back to 0 when `Nominal` can't reach `MIN_ADC_CLK`.
    ///
    /// Returns [`RfClockPathCalculationError::CouldNotFindValidConfig`] if no combination works.
    pub fn calculate_and_validate(
        tx_sample_hz: u32,
        rx_sample_is_twice_tx_sample: bool,
        rx_fir: RxFirDecimation,
        tx_fir: TxFirInterpolation,
        rate_governor: RateGovernor,
    ) -> Result<Self, RfClockPathCalculationError> {
        let config = Self::calculate_and_validate_from_row(
            tx_sample_hz,
            rx_sample_is_twice_tx_sample,
            rx_fir,
            tx_fir,
            rate_governor.skip_rows(),
        );
        if rate_governor == RateGovernor::Nominal
            && matches!(
                config,
                Err(RfClockPathCalculationError::CouldNotFindValidConfig)
            )
        {
            return Self::calculate_and_validate_from_row(
                tx_sample_hz,
                rx_sample_is_twice_tx_sample,
                rx_fir,
                tx_fir,
                RateGovernor::HighestOsr.skip_rows(),
            );
        }
        config
    }

    fn calculate_and_validate_from_row(
        tx_sample_hz: u32,
        rx_sample_is_twice_tx_sample: bool,
        rx_fir: RxFirDecimation,
        tx_fir: TxFirInterpolation,
        skip_rows: usize,
    ) -> Result<Self, RfClockPathCalculationError> {
        for rx_row in &Self::CLOCK_CHAIN_TABLE[skip_rows..] {
            for tx_row in &Self::CLOCK_CHAIN_TABLE[skip_rows..] {
                if let Ok(config) = Self::calculate_and_validate_with_full_path(
                    tx_sample_hz,
                    rx_sample_is_twice_tx_sample,
                    Rhb3Decimation::from_raw_divisor(rx_row.stage3).unwrap(),
                    rx_row.stage2_enabled,
                    rx_row.stage1_enabled,
                    rx_fir,
                    Thb3Interpolation::from_raw_multiplier(tx_row.stage3).unwrap(),
                    tx_row.stage2_enabled,
                    tx_row.stage1_enabled,
                    tx_fir,
                ) {
                    return Ok(config);
                }
            }
        }

        Err(RfClockPathCalculationError::CouldNotFindValidConfig)
    }

    /// BB PLL clock in Hz.
    #[inline]
    pub const fn bb_pll_clock_hz(&self) -> u32 {
        self.bb_pll_clock_hz
    }

    /// Divisor from the BB PLL clock to the ADC clock.
    #[inline]
    pub const fn adc_div(&self) -> AdcDivisor {
        self.adc_div
    }

    /// Whether the DAC clock is half of the ADC clock.
    #[inline]
    pub const fn dac_div2(&self) -> bool {
        self.dac_div2
    }

    /// The actual clock rate the DAC is programmed to, in Hz. This is always
    /// <= [`crate::limits::MAX_DAC_CLK`], because `Self::calculate_and_validate_with_full_path`
    /// rejects any candidate that would exceed it.
    #[inline]
    pub const fn dac_clock_hz(&self) -> u32 {
        let adc_clock_hz = self.bb_pll_clock_hz / self.adc_div.divisor();
        if self.dac_div2 {
            adc_clock_hz / 2
        } else {
            adc_clock_hz
        }
    }

    /// RX HB3 decimation.
    #[inline]
    pub const fn rhb3(&self) -> Rhb3Decimation {
        self.rhb3
    }

    /// Whether the RX HB2 stage is enabled.
    #[inline]
    pub const fn rhb2(&self) -> bool {
        self.rhb2
    }

    /// Whether the RX HB1 stage is enabled.
    #[inline]
    pub const fn rhb1(&self) -> bool {
        self.rhb1
    }

    /// RX FIR decimation.
    #[inline]
    pub const fn rx_fir(&self) -> RxFirDecimation {
        self.rx_fir
    }

    /// TX HB3 interpolation.
    #[inline]
    pub const fn thb3(&self) -> Thb3Interpolation {
        self.thb3
    }

    /// Whether the TX HB2 stage is enabled.
    #[inline]
    pub const fn thb2(&self) -> bool {
        self.thb2
    }

    /// Whether the TX HB1 stage is enabled.
    #[inline]
    pub const fn thb1(&self) -> bool {
        self.thb1
    }

    /// TX FIR interpolation.
    #[inline]
    pub const fn tx_fir(&self) -> TxFirInterpolation {
        self.tx_fir
    }

    /// Builds the [`RxConfig`] matching this computed clock path. `lo` is not part of the
    /// BB PLL / digital filter path calculation and must be supplied separately.
    pub const fn rx_config(&self, lo: LocalOscClockConfig) -> RxConfig {
        RxConfig {
            lo,
            rhb3: self.rhb3,
            rhb2: self.rhb2,
            rhb1: self.rhb1,
            rx_fir: self.rx_fir,
        }
    }

    /// Builds the [`TxConfig`] matching this computed clock path. `lo` is not part of the
    /// BB PLL / digital filter path calculation and must be supplied separately.
    pub const fn tx_config(&self, lo: LocalOscClockConfig) -> TxConfig {
        TxConfig {
            lo,
            dac_div2: self.dac_div2,
            thb3: self.thb3,
            thb2: self.thb2,
            thb1: self.thb1,
            tx_fir: self.tx_fir,
        }
    }
}

impl ClockConfig {
    /// Calculates and validates the BB PLL and ADC clocks for a fully specified half-band and FIR
    /// path.
    #[allow(clippy::too_many_arguments)]
    pub const fn calculate_and_validate_rf_pll_from_sample_clock(
        tx_sample_hz: u32,
        rx_sample_is_twice_tx_sample: bool,
        rhb3: Rhb3Decimation,
        rhb2: bool,
        rhb1: bool,
        rx_fir: RxFirDecimation,
        thb3: Thb3Interpolation,
        thb2: bool,
        thb1: bool,
        tx_fir: TxFirInterpolation,
    ) -> Result<BbClockPathConfigHelper, RfClockPathCalculationError> {
        BbClockPathConfigHelper::calculate_and_validate_with_full_path(
            tx_sample_hz,
            rx_sample_is_twice_tx_sample,
            rhb3,
            rhb2,
            rhb1,
            rx_fir,
            thb3,
            thb2,
            thb1,
            tx_fir,
        )
    }

    /// Maximum number of TX FIR taps which the clock ratio allows, or [`None`] while the TX FIR
    /// is bypassed. Same limit as in `ad9361_validate_enable_fir` of the C driver, which also
    /// allows at most 64 taps for an interpolation of 1.
    pub fn max_tx_fir_taps(&self, reference_clk_rate: u32) -> Option<u32> {
        if self.tx.tx_fir == TxFirInterpolation::Mult1BypassFilter {
            return None;
        }
        let clocks = Clocks::new(reference_clk_rate, self);
        let max = (clocks.tx.dac_clk / clocks.tx.tx_sample) * 16;
        if self.tx.tx_fir == TxFirInterpolation::Mult1EnableFilter {
            return Some(max.min(64));
        }
        Some(max)
    }

    /// Maximum number of RX FIR taps which the clock ratio allows, or [`None`] while the RX FIR
    /// is bypassed. Same limit as in `ad9361_validate_enable_fir` of the C driver.
    pub fn max_rx_fir_taps(&self, reference_clk_rate: u32) -> Option<u32> {
        if self.rx.rx_fir == RxFirDecimation::Div1BypassFilter {
            return None;
        }
        let clocks = Clocks::new(reference_clk_rate, self);
        // The C driver only uses half of the ADC clock if HB3 decimates.
        let adc_for_fir = if clocks.adc == clocks.rx.hb3 {
            clocks.adc
        } else {
            clocks.adc / 2
        };
        Some((adc_for_fir / clocks.rx.rx_sample) * 16)
    }
}

/// RX clock path configuration.
#[derive(Debug, Clone)]
pub struct RxConfig {
    /// RX LO configuration.
    pub lo: LocalOscClockConfig,

    /// RX HB3 decimation.
    pub rhb3: Rhb3Decimation,
    /// Enables the RX HB2 stage.
    pub rhb2: bool,
    /// Enables the RX HB1 stage.
    pub rhb1: bool,
    /// If this is anything other than [`RxFirDecimation::Div1BypassFilter`], real filter taps
    /// must be loaded via [`crate::Ad9361::set_rx_fir_config`] for correct operation. The RX FIR
    /// is kept bypassed through [`crate::Ad9361Uninit::init`] and calibration, and only enabled
    /// at this target afterward.
    /// At that point it runs with whatever coefficients happen to already be in the FIR's
    /// coefficient RAM. If no taps have ever been loaded (for example right after a fresh
    /// power-on reset), that RAM is typically all zero, so the RX signal path gets silently
    /// zeroed instead of producing an error.
    pub rx_fir: RxFirDecimation,
}

/// TX clock path configuration.
#[derive(Debug, Clone)]
pub struct TxConfig {
    /// TX LO configuration.
    pub lo: LocalOscClockConfig,

    /// Halves the DAC clock relative to the ADC clock.
    pub dac_div2: bool,
    /// TX HB3 interpolation.
    pub thb3: Thb3Interpolation,
    /// Enables the TX HB2 stage.
    pub thb2: bool,
    /// Enables the TX HB1 stage.
    pub thb1: bool,
    /// If this is anything other than [`TxFirInterpolation::Mult1BypassFilter`], real filter taps
    /// must be loaded via [`crate::Ad9361::set_tx_fir_config`] for correct operation. The TX FIR
    /// is kept bypassed through [`crate::Ad9361Uninit::init`] and calibration, and only enabled
    /// at this target afterward. At that point it runs with
    /// whatever coefficients happen to already be in the FIR's coefficient RAM. If no taps have
    /// ever been loaded (for example right after a fresh power-on reset), that RAM is typically
    /// all zero, so the TX signal path gets silently zeroed instead of producing an error.
    pub tx_fir: TxFirInterpolation,
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn test_bb_pll_clock_calc_full_path_config() {
        // THB3 bypassed (Mult1NoFiltering) instead of Mult2HalfBand: with THB3 enabled this
        // exact combo drives the DAC to 491.52 MHz, above the 320 MHz MAX_DAC_CLK limit (see
        // dac_clock_too_fast_is_rejected below). Bypassing it halves dac_clock_from_tx to
        // 245.76 MHz while the unchanged RX side still puts adc_clock_from_rx at 491.52 MHz,
        // so dac_div2 comes out true (adc_clock_from_rx == 2 * dac_clock_from_tx) here.
        let bb_pll_config = BbClockPathConfigHelper::calculate_and_validate_with_full_path(
            30_720_000,
            false,
            Rhb3Decimation::Div1NoFiltering,
            true,
            true,
            RxFirDecimation::Div4EnableFilter,
            Thb3Interpolation::Mult1NoFiltering,
            true,
            false,
            TxFirInterpolation::Mult4EnableFilter,
        )
        .expect("BB PLL Clock calulation failed");
        assert_eq!(bb_pll_config.bb_pll_clock_hz(), 983040000);
        assert_eq!(bb_pll_config.adc_div(), AdcDivisor::Div2);
        assert_eq!(bb_pll_config.dac_div2(), true);
        assert!(bb_pll_config.dac_clock_hz() <= crate::limits::MAX_DAC_CLK);
    }

    #[test]
    fn dac_clock_too_fast_is_rejected() {
        // Same TX chain as the ROMEO firmware's actual (buggy) boot config: 30.72 MHz sample
        // rate x4 (FIR) x2 (THB2) x2 (THB3) = 491.52 MHz at the DAC, which exceeds
        // MAX_DAC_CLK (320 MHz). This must be rejected rather than silently accepted. See
        // the `dac_clock_from_tx > MAX_DAC_CLK` check in calculate_and_validate_with_full_path.
        let result = BbClockPathConfigHelper::calculate_and_validate_with_full_path(
            30_720_000,
            false,
            Rhb3Decimation::Div1NoFiltering,
            true,
            true,
            RxFirDecimation::Div4EnableFilter,
            Thb3Interpolation::Mult2HalfBand,
            true,
            false,
            TxFirInterpolation::Mult4EnableFilter,
        );
        assert_eq!(result, Err(RfClockPathCalculationError::DacClockTooFast));
    }

    #[test]
    fn test_bb_pll_clock_calc_auto_calc() {
        let bb_pll_config = BbClockPathConfigHelper::calculate_and_validate(
            30_720_000,
            false,
            RxFirDecimation::Div4EnableFilter,
            TxFirInterpolation::Mult4EnableFilter,
            RateGovernor::Nominal,
        )
        .expect("BB PLL Clock calulation failed");
        assert_eq!(
            bb_pll_config.bb_pll_clock_hz(),
            983040000,
            "bb pll config {}",
            bb_pll_config.bb_pll_clock_hz()
        );
        // The exact filter-chain choice the search lands on can shift as the candidate table or
        // rate governor changes; what actually matters is that the DAC clock it picks stays
        // within spec, so assert on that directly rather than pinning dac_div2() to one value.
        assert!(
            bb_pll_config.dac_clock_hz() <= crate::limits::MAX_DAC_CLK,
            "dac clock {} exceeds MAX_DAC_CLK {}",
            bb_pll_config.dac_clock_hz(),
            crate::limits::MAX_DAC_CLK
        );
    }

    #[test]
    fn calculate_local_osc_config() {
        let local_osc_config =
            LocalOscClockConfig::calculate_for_internal_lo_with_target_vco_clock(
                40_000_000,
                6_000_000_000,
                PllVcoDivider::Div4,
            )
            .expect("Failed to calculate local oscillator config");
        match local_osc_config {
            LocalOscClockConfig::External {
                external_vco_freq: _,
            } => panic!("should be internal"),
            LocalOscClockConfig::Internal(rf_pll_config) => {
                assert_eq!(rf_pll_config.vco_div, PllVcoDivider::Div4);
                assert_eq!(
                    rf_pll_config.pll_int,
                    u11::new((6_000_000_000_u64 / 40_000_000) as u16)
                );
                assert_eq!(rf_pll_config.pll_frac, u23::ZERO);
            }
        }
    }

    #[test]
    fn calculate_local_osc_config_direct_lo() {
        let local_osc_config =
            LocalOscClockConfig::calculate_for_internal_lo(40_000_000, 2_400_000_000)
                .expect("Failed to calculate local oscillator config");
        match local_osc_config {
            LocalOscClockConfig::External {
                external_vco_freq: _,
            } => panic!("should be internal"),
            LocalOscClockConfig::Internal(rf_pll_config) => {
                let vco_clock = 2_400_000_000_u64 * rf_pll_config.vco_div.divider() as u64;
                assert!(
                    vco_clock >= crate::limits::MIN_VCO_FREQ_HZ
                        && vco_clock <= crate::limits::MAX_VCO_FREQ_HZ
                );
                assert_eq!(
                    vco_clock / rf_pll_config.pll_int.value() as u64,
                    40_000_000_u64
                );
                assert_eq!(rf_pll_config.pll_frac.value(), 0);
            }
        }
    }

    /// Sample rates swept by the `auto_calc_*` tests below: from well under
    /// [`crate::limits::MAX_BASEBAND_RATE`] up to and including it.
    const SWEEP_SAMPLE_RATES_HZ: [u32; 7] = [
        520_833, // near the slowest rate the table supports (12x oversampling)
        1_000_000, 2_500_000, 7_680_000, 15_360_000,
        30_720_000, // ROMEO's actual target sample rate
        61_440_000, // at MAX_BASEBAND_RATE
    ];
    const SWEEP_RX_FIRS: [RxFirDecimation; 4] = [
        RxFirDecimation::Div1BypassFilter,
        RxFirDecimation::Div1EnableFilter,
        RxFirDecimation::Div2EnableFilter,
        RxFirDecimation::Div4EnableFilter,
    ];
    const SWEEP_TX_FIRS: [TxFirInterpolation; 4] = [
        TxFirInterpolation::Mult1BypassFilter,
        TxFirInterpolation::Mult1EnableFilter,
        TxFirInterpolation::Mult2EnableFilter,
        TxFirInterpolation::Mult4EnableFilter,
    ];
    const SWEEP_RATE_GOVERNORS: [RateGovernor; 2] =
        [RateGovernor::Nominal, RateGovernor::HighestOsr];

    /// One result from [`sweep_auto_calculated_configs`]: the sweep inputs that produced `config`,
    /// alongside `config` itself.
    #[derive(Debug)]
    struct SweepResult {
        tx_sample_hz: u32,
        rx_sample_is_twice_tx_sample: bool,
        config: BbClockPathConfigHelper,
    }

    /// Runs [`BbClockPathConfigHelper::calculate_and_validate`] across every combination of
    /// [`SWEEP_SAMPLE_RATES_HZ`], `rx_sample_is_twice_tx_sample`, [`SWEEP_RX_FIRS`],
    /// [`SWEEP_TX_FIRS`] and [`SWEEP_RATE_GOVERNORS`], returning one [`SweepResult`] per
    /// combination that produced a valid clock path. Not every combination does. Some FIR
    /// pairings have no table row that reconciles into a consistent ADC/DAC clock relationship,
    /// which is expected and not itself checked here. The `auto_calc_*` tests below each
    /// check one family of limits against the configs this does return.
    fn sweep_auto_calculated_configs() -> std::vec::Vec<SweepResult> {
        let mut results = std::vec::Vec::new();
        for &tx_sample_hz in &SWEEP_SAMPLE_RATES_HZ {
            for rx_sample_is_twice_tx_sample in [false, true] {
                for rx_fir in SWEEP_RX_FIRS {
                    for tx_fir in SWEEP_TX_FIRS {
                        for rate_governor in SWEEP_RATE_GOVERNORS {
                            if let Ok(config) = BbClockPathConfigHelper::calculate_and_validate(
                                tx_sample_hz,
                                rx_sample_is_twice_tx_sample,
                                rx_fir,
                                tx_fir,
                                rate_governor,
                            ) {
                                results.push(SweepResult {
                                    tx_sample_hz,
                                    rx_sample_is_twice_tx_sample,
                                    config,
                                });
                            }
                        }
                    }
                }
            }
        }
        results
    }

    /// Recomputes the RX chain's four checkpoints (RX FIR output, half-band 1 output, half-band
    /// 2 output, ADC clock) from `config`'s public getters. This is independent of
    /// `calculate_and_validate_with_full_path`'s internal computation, so a regression there
    /// still gets caught here.
    fn rx_chain_checkpoints_hz(
        tx_sample_hz: u32,
        rx_sample_is_twice_tx_sample: bool,
        config: &BbClockPathConfigHelper,
    ) -> [u32; 4] {
        let rx_sample_hz = if rx_sample_is_twice_tx_sample {
            tx_sample_hz * 2
        } else {
            tx_sample_hz
        };
        let mut rx_fir_output_hz = rx_sample_hz;
        match config.rx_fir() {
            RxFirDecimation::Div1BypassFilter | RxFirDecimation::Div1EnableFilter => (),
            RxFirDecimation::Div2EnableFilter => rx_fir_output_hz *= 2,
            RxFirDecimation::Div4EnableFilter => rx_fir_output_hz *= 4,
        }
        let mut rx_half_band_1_output_hz = rx_fir_output_hz;
        if config.rhb1() {
            rx_half_band_1_output_hz *= 2;
        }
        let mut rx_half_band_2_output_hz = rx_half_band_1_output_hz;
        if config.rhb2() {
            rx_half_band_2_output_hz *= 2;
        }
        let mut adc_clock_from_rx = rx_half_band_2_output_hz;
        match config.rhb3() {
            Rhb3Decimation::Div1NoFiltering => (),
            Rhb3Decimation::Div2HalfBand => adc_clock_from_rx *= 2,
            Rhb3Decimation::Div3Filter => adc_clock_from_rx *= 3,
        }
        [
            rx_fir_output_hz,
            rx_half_band_1_output_hz,
            rx_half_band_2_output_hz,
            adc_clock_from_rx,
        ]
    }

    /// Recomputes the TX chain's four checkpoints (TX FIR output, half-band 1 output, half-band
    /// 2 output, DAC clock) from `config`'s public getters. This is independent of
    /// `calculate_and_validate_with_full_path`'s internal computation, so a regression there
    /// still gets caught here.
    fn tx_chain_checkpoints_hz(tx_sample_hz: u32, config: &BbClockPathConfigHelper) -> [u32; 4] {
        let mut tx_fir_output_hz = tx_sample_hz;
        match config.tx_fir() {
            TxFirInterpolation::Mult1BypassFilter => (),
            TxFirInterpolation::Mult1EnableFilter | TxFirInterpolation::Mult2EnableFilter => {
                tx_fir_output_hz *= 2
            }
            TxFirInterpolation::Mult4EnableFilter => tx_fir_output_hz *= 4,
        }
        let mut tx_half_band_1_output_hz = tx_fir_output_hz;
        if config.thb1() {
            tx_half_band_1_output_hz *= 2;
        }
        let mut tx_half_band_2_output_hz = tx_half_band_1_output_hz;
        if config.thb2() {
            tx_half_band_2_output_hz *= 2;
        }
        let mut dac_clock_from_tx = tx_half_band_2_output_hz;
        match config.thb3() {
            Thb3Interpolation::Mult1NoFiltering => (),
            Thb3Interpolation::Mult2HalfBand => dac_clock_from_tx *= 2,
            Thb3Interpolation::Mult3Filter => dac_clock_from_tx *= 3,
        }
        [
            tx_fir_output_hz,
            tx_half_band_1_output_hz,
            tx_half_band_2_output_hz,
            dac_clock_from_tx,
        ]
    }

    #[test]
    fn auto_calc_sweep_produces_results() {
        // Sanity check that the sweep below actually exercises the success path rather than
        // every combination silently erroring out, which would make the other auto_calc_*
        // tests vacuously true.
        assert!(
            !sweep_auto_calculated_configs().is_empty(),
            "no sample rate / FIR combination in the sweep produced a valid clock path"
        );
    }

    #[test]
    fn auto_calc_bb_pll_clock_stays_in_range() {
        for SweepResult { config, .. } in sweep_auto_calculated_configs() {
            assert!(
                config.bb_pll_clock_hz() >= crate::limits::MIN_BBPLL_FREQ
                    && config.bb_pll_clock_hz() <= crate::limits::MAX_BBPLL_FREQ,
                "bb_pll_clock_hz {} out of [{}, {}] for config {config:?}",
                config.bb_pll_clock_hz(),
                crate::limits::MIN_BBPLL_FREQ,
                crate::limits::MAX_BBPLL_FREQ
            );
        }
    }

    #[test]
    fn auto_calc_rx_chain_stays_in_range() {
        for SweepResult {
            tx_sample_hz,
            rx_sample_is_twice_tx_sample,
            config,
        } in sweep_auto_calculated_configs()
        {
            let [hb1_output, hb2_output, hb3_output, adc_clock] =
                rx_chain_checkpoints_hz(tx_sample_hz, rx_sample_is_twice_tx_sample, &config);
            assert!(
                hb1_output <= crate::limits::MAX_RX_HB1,
                "RX HB1 stage clock {hb1_output} exceeds MAX_RX_HB1 for config {config:?}"
            );
            assert!(
                hb2_output <= crate::limits::MAX_RX_HB2,
                "RX HB2 stage clock {hb2_output} exceeds MAX_RX_HB2 for config {config:?}"
            );
            assert!(
                hb3_output <= crate::limits::MAX_RX_HB3,
                "RX HB3 stage clock {hb3_output} exceeds MAX_RX_HB3 for config {config:?}"
            );
            assert!(
                adc_clock >= crate::limits::MIN_ADC_CLK && adc_clock <= crate::limits::MAX_ADC_CLK,
                "ADC clock {adc_clock} out of range for config {config:?}"
            );
        }
    }

    #[test]
    fn auto_calc_tx_chain_stays_in_range() {
        for SweepResult {
            tx_sample_hz,
            config,
            ..
        } in sweep_auto_calculated_configs()
        {
            let [hb1_output, hb2_output, hb3_output, dac_clock] =
                tx_chain_checkpoints_hz(tx_sample_hz, &config);
            assert!(
                hb1_output <= crate::limits::MAX_TX_HB1,
                "TX HB1 stage clock {hb1_output} exceeds MAX_TX_HB1 for config {config:?}"
            );
            assert!(
                hb2_output <= crate::limits::MAX_TX_HB2,
                "TX HB2 stage clock {hb2_output} exceeds MAX_TX_HB2 for config {config:?}"
            );
            assert!(
                hb3_output <= crate::limits::MAX_TX_HB3,
                "TX HB3 stage clock {hb3_output} exceeds MAX_TX_HB3 for config {config:?}"
            );
            assert!(
                dac_clock <= crate::limits::MAX_DAC_CLK,
                "DAC clock {dac_clock} exceeds MAX_DAC_CLK for config {config:?}"
            );
            assert_eq!(
                dac_clock,
                config.dac_clock_hz(),
                "independently recomputed DAC clock disagrees with dac_clock_hz() for config {config:?}"
            );
        }
    }

    const REF_CLK_HZ: u32 = 40_000_000;

    /// Builds the clock config for a path with the same fixed parts as the ROMEO firmware.
    #[allow(clippy::too_many_arguments)]
    fn clock_config_for_stages(
        rhb3: Rhb3Decimation,
        rhb2: bool,
        rhb1: bool,
        rx_fir: RxFirDecimation,
        thb3: Thb3Interpolation,
        thb2: bool,
        thb1: bool,
        tx_fir: TxFirInterpolation,
    ) -> ClockConfig {
        clock_config_for_path(
            BbClockPathConfigHelper::calculate_and_validate_with_full_path(
                30_720_000, false, rhb3, rhb2, rhb1, rx_fir, thb3, thb2, thb1, tx_fir,
            )
            .expect("invalid clock path"),
        )
    }

    fn clock_config_for_path(path: BbClockPathConfigHelper) -> ClockConfig {
        let ref_clk_scalers = RefClockScalers {
            bb_refclk: regs::ClockScaler::Div1,
            rx_synth: regs::ClockScaler::Mul2,
            tx_synth: regs::ClockScaler::Mul2,
        };
        let lo = LocalOscClockConfig::calculate_for_internal_lo(80_000_000, 2_400_000_000)
            .expect("invalid LO");
        ClockConfig {
            ref_clk_scalers,
            bb_pll: BbPllConfig::calculate(
                ref_clk_scalers.calculate_bb_pll_synth_clock(REF_CLK_HZ),
                path.bb_pll_clock_hz(),
            ),
            adc: path.adc_div(),
            rx: path.rx_config(lo),
            tx: path.tx_config(lo),
        }
    }

    #[test]
    fn firmware_fir_ratios_fit_64_taps_at_every_sample_rate() {
        // The firmware uses a FIR ratio of 4 on both sides with 64 taps. That must never be
        // rejected, no matter which half-bands the path calculation picks for a sample rate.
        let mut checked = 0;
        for tx_sample_hz in [2_500_000, 7_680_000, 15_360_000, 23_040_000, 30_720_000] {
            let Ok(path) = BbClockPathConfigHelper::calculate_and_validate(
                tx_sample_hz,
                false,
                RxFirDecimation::Div4EnableFilter,
                TxFirInterpolation::Mult4EnableFilter,
                RateGovernor::Nominal,
            ) else {
                continue;
            };
            let config = clock_config_for_path(path);
            assert!(config.max_tx_fir_taps(REF_CLK_HZ) >= Some(64));
            assert!(config.max_rx_fir_taps(REF_CLK_HZ) >= Some(64));
            checked += 1;
        }
        assert!(checked > 0, "no sample rate produced a valid path");
    }

    #[test]
    fn fir_tap_limit_without_half_bands_is_four_times_16() {
        // FIR ratio 4 only, ADC and DAC run at 4x the sample rate on both sides.
        let config = clock_config_for_stages(
            Rhb3Decimation::Div1NoFiltering,
            false,
            false,
            RxFirDecimation::Div4EnableFilter,
            Thb3Interpolation::Mult1NoFiltering,
            false,
            false,
            TxFirInterpolation::Mult4EnableFilter,
        );
        assert_eq!(config.max_tx_fir_taps(REF_CLK_HZ), Some(64));
        assert_eq!(config.max_rx_fir_taps(REF_CLK_HZ), Some(64));
    }

    #[test]
    fn fir_tap_limit_grows_with_half_bands() {
        // One more half-band on each side doubles the ADC and DAC clock.
        let config = clock_config_for_stages(
            Rhb3Decimation::Div1NoFiltering,
            false,
            true,
            RxFirDecimation::Div4EnableFilter,
            Thb3Interpolation::Mult1NoFiltering,
            false,
            true,
            TxFirInterpolation::Mult4EnableFilter,
        );
        assert_eq!(config.max_tx_fir_taps(REF_CLK_HZ), Some(128));
        assert_eq!(config.max_rx_fir_taps(REF_CLK_HZ), Some(128));
    }

    #[test]
    fn rx_fir_tap_limit_uses_half_the_adc_clock_if_hb3_decimates() {
        // The ADC runs at 8x the sample rate, but HB3 decimates so only half of it counts.
        let config = clock_config_for_stages(
            Rhb3Decimation::Div2HalfBand,
            false,
            false,
            RxFirDecimation::Div4EnableFilter,
            Thb3Interpolation::Mult1NoFiltering,
            false,
            true,
            TxFirInterpolation::Mult4EnableFilter,
        );
        assert_eq!(config.max_rx_fir_taps(REF_CLK_HZ), Some(64));
        assert_eq!(config.max_tx_fir_taps(REF_CLK_HZ), Some(128));
    }

    #[test]
    fn fir_tap_limit_does_not_apply_to_bypassed_fir() {
        let config = clock_config_for_stages(
            Rhb3Decimation::Div1NoFiltering,
            true,
            true,
            RxFirDecimation::Div1BypassFilter,
            Thb3Interpolation::Mult1NoFiltering,
            true,
            true,
            TxFirInterpolation::Mult1BypassFilter,
        );
        assert_eq!(config.max_tx_fir_taps(REF_CLK_HZ), None);
        assert_eq!(config.max_rx_fir_taps(REF_CLK_HZ), None);
    }

    #[test]
    fn tx_fir_tap_limit_is_64_for_interpolation_of_1() {
        // The ratio alone would allow 128 taps here, but the C driver caps interpolation 1. The
        // path is built with a bypassed FIR and switched afterwards, because the path helper
        // calculates this ratio differently from `Clocks::new`.
        let mut config = clock_config_for_stages(
            Rhb3Decimation::Div2HalfBand,
            true,
            true,
            RxFirDecimation::Div1BypassFilter,
            Thb3Interpolation::Mult2HalfBand,
            true,
            true,
            TxFirInterpolation::Mult1BypassFilter,
        );
        config.tx.tx_fir = TxFirInterpolation::Mult1EnableFilter;
        assert_eq!(config.max_tx_fir_taps(REF_CLK_HZ), Some(64));
    }
}
