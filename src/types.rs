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
    /// RX channel 1.
    Rx1,
    /// RX channel 2.
    Rx2,
}
/// Identifies one of the two TX channels.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TransmitterId {
    /// TX channel 1.
    Tx1,
    /// TX channel 2.
    Tx2,
}
/// Gain control mode of an RX channel.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GainControlMode {
    /// The gain is set by software.
    Manual,
    /// Automatic gain control in fast attack mode.
    AutoFastAttack,
    /// Automatic gain control in slow attack mode.
    AutoSlowAttack,
    /// Automatic gain control in hybrid mode.
    AutoHybrid,
}
/// Gain index the fast AGC goes to on an event, like leaving the RX state or the EN_AGC pin
/// going high.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FastAgcTargetGainIndexType {
    /// Go to the maximum gain index.
    MaxGain,
    /// Go to the set gain index.
    SetGain,
    /// Go to the optimized gain index.
    OptimizedGain,
    /// Keep the current gain index.
    NoGainChange,
}
/// Clocks of the RX chain, from the BB PLL down to the sample rate.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RxClock {
    /// BB PLL clock.
    BbPll,
    /// ADC clock.
    Adc,
    /// Output of the RX HB3 stage.
    R2,
    /// Output of the RX HB2 stage.
    R1,
    /// Output of the RX HB1 stage.
    ClkRf,
    /// RX sample rate.
    RxSample,
}

impl RxClock {
    /// Number of clocks in the RX chain.
    pub const NUM_CLOCKS: usize = 6;

    /// Maximum frequency of the clock in Hz.
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
/// Clocks of the TX chain, from the DAC clock down to the sample rate.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TxClock {
    /// Slot of the BB PLL, which the TX chain does not use. Keeps the order of [`RxClock`].
    Ignore,
    /// DAC clock.
    Dac,
    /// Input of the TX HB3 stage.
    T2,
    /// Input of the TX HB2 stage.
    T1,
    /// Input of the TX HB1 stage.
    ClkTf,
    /// TX sample rate.
    TxSample,
}

impl TxClock {
    /// Number of clocks in the TX chain.
    pub const NUM_CLOCKS: usize = 6;

    /// Maximum frequency of the clock in Hz.
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
