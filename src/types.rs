use crate::limits;
use arbitrary_int::u5;

/// How TX quadrature calibration picks the RX NCO phase it uses to observe the TX calibration
/// tone. `Manual`/`Calculated` both try one specific phase first and only fall back to a full
/// sweep across all candidate phases if that doesn't converge; `ForceSearch` always does the
/// full sweep.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RxPhaseConfig {
    /// Try this exact phase value first.
    Manual(u5),
    /// Try the phase derived from the configured RX/TX clock relationship first.
    Calculated,
    /// Skip straight to sweeping all candidate phases and picking whichever gives the best
    /// calibration result.
    ForceSearch,
}

/// Identifies one of the two RX channels.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ReceiverId {
    Rx1,
    Rx2,
}
/// Identifies one of the two TX channels.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TransmitterId {
    Tx1,
    Tx2,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GainControlMode {
    Manual,
    AutoFastAttack,
    AutoSlowAttack,
    AutoHybrid,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FastAgcTargetGainIndexType {
    MaxGain,
    SetGain,
    OptimizedGain,
    NoGainChange,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RxClock {
    BbPll,
    Adc,
    R2,
    R1,
    ClkRf,
    RxSample,
}

impl RxClock {
    pub const NUM_CLOCKS: usize = 6;

    #[inline]
    pub const fn limit_hz(&self) -> u32 {
        match self {
            RxClock::BbPll => limits::MAX_BBPLL_FREQ,
            RxClock::Adc => limits::MAX_ADC_CLK,
            RxClock::R2 => limits::MAX_RX_HB3,
            RxClock::R1 => limits::MAX_RX_HB2,
            RxClock::ClkRf => limits::MAX_RX_HB1,
            RxClock::RxSample => limits::MAX_BASEBAND_RATE,
        }
    }
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TxClock {
    Ignore,
    Dac,
    T2,
    T1,
    ClkTf,
    TxSample,
}

impl TxClock {
    pub const NUM_CLOCKS: usize = 6;

    #[inline]
    pub const fn limit_hz(&self) -> u32 {
        match self {
            TxClock::Ignore => limits::MAX_BBPLL_FREQ,
            TxClock::Dac => limits::MAX_DAC_CLK,
            TxClock::T2 => limits::MAX_TX_HB3,
            TxClock::T1 => limits::MAX_TX_HB2,
            TxClock::ClkTf => limits::MAX_TX_HB1,
            TxClock::TxSample => limits::MAX_BASEBAND_RATE,
        }
    }
}
