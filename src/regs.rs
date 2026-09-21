pub use arbitrary_int::u10;

pub use clk_bb_pll::{AdcDivisor, ClkOutMode};
pub use clock_enable::ExternalClockConfig;
pub use dec_power_measure_duration::DecPowerMeasurementSource;
pub use rfpll_dividers::PllVcoDividerBits;
pub use state::EnsmState;

#[bitbybit::bitenum(u2, exhaustive = true)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum ClockScaler {
    Div1 = 0b00,
    Div2 = 0b01,
    Div4 = 0b10,
    Mul2 = 0b11,
}

impl ClockScaler {
    pub const fn mult(&self) -> u32 {
        match self {
            ClockScaler::Div1 => 1,
            ClockScaler::Div2 => 1,
            ClockScaler::Div4 => 1,
            ClockScaler::Mul2 => 2,
        }
    }

    pub const fn div(&self) -> u32 {
        match self {
            ClockScaler::Div1 => 1,
            ClockScaler::Div2 => 2,
            ClockScaler::Div4 => 4,
            ClockScaler::Mul2 => 1,
        }
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum Register {
    /// SPI Configuration
    SpiConf = 0x000,
    /// Multi-Chip Sync and Tx Mon Control
    MultichipSyncAndTxMonCtrl = 0x001,
    /// Tx Enable & Filter Control
    TxEnableFilterCtrl = 0x002,
    /// Rx Enable & Filter Control
    RxEnableFilterCtrl = 0x003,
    /// Input Select
    InputSelect = 0x004,
    /// RFPLL Dividers
    RfPllDividers = 0x005,
    /// Rx Clock & Data Delay
    RxClockDataDelay = 0x006,
    /// Tx Clock & Data Delay
    TxClockDataDelay = 0x007,
    /// Clock Enable
    ClockEnable = 0x009,
    /// BBPLL
    BbPll = 0x00A,
    /// Offset
    TempOffset = 0x00B,
    /// Start Temp Reading
    StartTempReading = 0x00C,
    /// Temp Sense2
    TempSense2 = 0x00D,
    /// Temperature
    Temperature = 0x00E,
    /// Temp Sensor Config
    TempSensorConfig = 0x00F,
    /// Parallel Port Configuration 1
    ParallelPortConf1 = 0x010,
    /// Parallel Port Configuration 2
    ParallelPortConf2 = 0x011,
    /// Parallel Port Configuration 3
    ParallelPortConf3 = 0x012,
    /// ENSM Mode
    EnsmMode = 0x013,
    /// ENSM Config 1
    EnsmConfig1 = 0x014,
    /// ENSM Config 2
    EnsmConfig2 = 0x015,
    /// Calibration Control
    CalibrationCtrl = 0x016,
    /// State
    State = 0x017,
    /// AuxDAC 1 Word
    Auxdac1Word = 0x018,
    /// AuxDAC 2 Word
    Auxdac2Word = 0x019,
    /// AuxDAC 1 Config
    Auxdac1Config = 0x01A,
    /// AuxDAC 2 Config
    Auxdac2Config = 0x01B,
    /// AuxADC Clock Divider
    AuxadcClockDivider = 0x01C,
    /// Aux ADC Config
    AuxadcConfig = 0x01D,
    /// AuxADC Word MSB
    AuxadcWordMsb = 0x01E,
    /// AuxADC LSB
    AuxadcLsb = 0x01F,
    /// Auto GPO
    AutoGpo = 0x020,
    /// AGC Gain Lock Delay
    AgcGainLockDelay = 0x021,
    /// AGC Attack Delay
    AgcAttackDelay = 0x022,
    /// AuxDAC Enable Control
    AuxdacEnableCtrl = 0x023,
    /// RX Load Synth Delay
    RxLoadSynthDelay = 0x024,
    /// TX Load Synth Delay
    TxLoadSynthDelay = 0x025,
    /// External LNA control
    ExternalLnaCtrl = 0x026,
    /// GPO Force and Init
    GpoForceAndInit = 0x027,
    /// GPO0 Rx delay
    Gpo0RxDelay = 0x028,
    /// GPO1 Rx delay
    Gpo1RxDelay = 0x029,
    /// GPO2 Rx delay
    Gpo2RxDelay = 0x02A,
    /// GPO3 Rx delay
    Gpo3RxDelay = 0x02B,
    /// GPO0 Tx Delay
    Gpo0TxDelay = 0x02C,
    /// GPO1 Tx Delay
    Gpo1TxDelay = 0x02D,
    /// GPO2 Tx Delay
    Gpo2TxDelay = 0x02E,
    /// GPO3 Tx Delay
    Gpo3TxDelay = 0x02F,
    /// AuxDAC1 Rx Delay
    Auxdac1RxDelay = 0x030,
    /// AuxDAC1 Tx Delay
    Auxdac1TxDelay = 0x031,
    /// AuxDAC2 Rx Delay
    Auxdac2RxDelay = 0x032,
    /// AuxDAC2 Tx Delay
    Auxdac2TxDelay = 0x033,
    /// Control Output Pointer
    CtrlOutputPointer = 0x035,
    /// Control Output Enable
    CtrlOutputEnable = 0x036,
    /// Product ID
    ProductId = 0x037,
    /// Reference Clock Cycles
    ReferenceClockCycles = 0x03A,
    /// Digital I/O Control
    DigitalIoCtrl = 0x03B,
    /// LVDS Bias control
    LvdsBiasCtrl = 0x03C,
    /// LVDS Invert control1
    LvdsInvertCtrl1 = 0x03D,
    /// LVDS Invert control2
    LvdsInvertCtrl2 = 0x03E,
    /// BB PLL Control 1
    BbPllCtrl1 = 0x03F,
    /// Fractional BB Freq Word 1
    FractBbFreqWord1 = 0x041,
    /// Fractional BB Freq Word 2
    FractBbFreqWord2 = 0x042,
    /// Fractional BB Freq Word 3
    FractBbFreqWord3 = 0x043,
    /// Integer BB Freq Word
    IntegerBbFreqWord = 0x044,
    /// Clock Control
    ClockCtrl = 0x045,
    /// CP Current
    CpCurrent = 0x046,
    /// CP Bleed Current
    CpBleedCurrent = 0x047,
    /// Loop Filter 1
    LoopFilter1 = 0x048,
    /// Loop Filter 2
    LoopFilter2 = 0x049,
    /// Loop Filter 3
    LoopFilter3 = 0x04A,
    /// VCO Control
    VcoCtrl = 0x04B,
    VcoProgram1 = 0x04C,
    VcoProgram2 = 0x04D,
    /// SDM Control
    SdmCtrl = 0x04E,
    /// Rx Synth Power Down Override
    RxSynthPowerDownOverride = 0x050,
    /// TX Synth Power Down Override
    TxSynthPowerDownOverride = 0x051,
    /// Rx Analog Power Down Override 1
    RxAnalogPowerDownOverride1 = 0x052,
    /// Rx Analog Power Down Override 2
    RxAnalogPowerDownOverride2 = 0x053,
    /// Rx1 ADC Power Down Override
    Rx1AdcPowerDownOverride = 0x054,
    /// Rx2 ADC Power Down Override
    Rx2AdcPowerDownOverride = 0x055,
    /// Tx Analog Power Down Override 1
    TxAnalogPowerDownOverride1 = 0x056,
    /// Analog Power Down Override
    AnalogPowerDownOverride = 0x057,
    /// Misc Power Down Override
    MiscPowerDownOverride = 0x058,
    /// CH 1 Overflow
    Ch1Overflow = 0x05E,
    /// CH 2 Overflow
    Ch2Overflow = 0x05F,
    /// TX Filter Coefficient Address
    TxFilterCoefAddr = 0x060,
    /// TX Filter Coefficient Write Data 1
    TxFilterCoefWriteData1 = 0x061,
    /// TX Filter Coefficient Write Data 2
    TxFilterCoefWriteData2 = 0x062,
    /// TX Filter Coefficient Read Data 1
    TxFilterCoefReadData1 = 0x063,
    /// TX Filter Coefficient Read Data 2
    TxFilterCoefReadData2 = 0x064,
    /// TX Filter Configuration
    TxFilterConf = 0x065,
    /// Tx Mon Low Gain
    TxMonLowGain = 0x067,
    /// Tx Mon High Gain
    TxMonHighGain = 0x068,
    /// Tx Mon Delay
    TxMonDelay = 0x069,
    /// Tx Level Threshold
    TxLevelThresh = 0x06A,
    /// TX RSSI1
    TxRssi1 = 0x06B,
    /// TX RSSI2
    TxRssi2 = 0x06C,
    /// TX RSSI LSB
    TxRssiLsb = 0x06D,
    /// TPM Mode Enable
    TpmModeEnable = 0x06E,
    /// Temp Gain Coefficient
    TxMonTempGainCoef = 0x06F,
    /// Tx Mon 1 Config
    TxMon1Config = 0x070,
    /// Tx Mon 2 Config
    TxMon2Config = 0x071,
    /// Tx1 Atten 0
    Tx1Atten0 = 0x073,
    /// Tx1 Atten 1
    Tx1Atten1 = 0x074,
    /// Tx2 Atten 0
    Tx2Atten0 = 0x075,
    /// Tx2 Atten 1
    Tx2Atten1 = 0x076,
    /// Tx Atten Offset
    TxAttenOffset = 0x077,
    /// Tx Atten Threshold
    TxAttenThresh = 0x078,
    /// Tx1 Dig Attenuation
    Tx1DigAtten = 0x079,
    /// Tx2 Dig Attenuation
    Tx2DigAtten = 0x07C,
    /// TX1 Symbol Attenuation
    Tx1SymbolAtten = 0x07F,
    /// TX2 Symbol Attenuation
    Tx2SymbolAtten = 0x080,
    /// TX Symbol Atten Config
    TxSymbolAttenConfig = 0x081,
    /// Tx1 Out 1 Phase Corr
    Tx1Out1PhaseCorr = 0x08E,
    /// Tx1 Out 1 Gain Corr
    Tx1Out1GainCorr = 0x08F,
    /// Tx2 Out 1 Phase Corr
    Tx2Out1PhaseCorr = 0x090,
    /// Tx2 Out 1 Gain Corr
    Tx2Out1GainCorr = 0x091,
    /// Tx1 Out 1 Offset I
    Tx1Out1OffsetI = 0x092,
    /// Tx1 Out 1 Offset Q
    Tx1Out1OffsetQ = 0x093,
    /// Tx2 Out 1 Offset I
    Tx2Out1OffsetI = 0x094,
    /// Tx2 Out 1 Offset Q
    Tx2Out1OffsetQ = 0x095,
    /// Tx1 Out 2 Phase Corr
    Tx1Out2PhaseCorr = 0x096,
    /// Tx1 Out 2 Gain Corr
    Tx1Out2GainCorr = 0x097,
    /// Tx2 Out 2 Phase Corr
    Tx2Out2PhaseCorr = 0x098,
    /// Tx2 Out 2 Gain Corr
    Tx2Out2GainCorr = 0x099,
    /// Tx1 Out 2 Offset I
    Tx1Out2OffsetI = 0x09A,
    /// Tx1 Out 2 Offset Q
    Tx1Out2OffsetQ = 0x09B,
    /// Tx2 Out 2 Offset I
    Tx2Out2OffsetI = 0x09C,
    /// Tx2 Out 2 Offset Q
    Tx2Out2OffsetQ = 0x09D,
    /// Force Bits
    TxForceBits = 0x09F,
    /// Quad Cal NCO Freq & Phase Offset
    QuadCalNcoFreqPhaseOffset = 0x0A0,
    /// Quad Cal Control
    QuadCalCtrl = 0x0A1,
    /// Kexp 1
    Kexp1 = 0x0A2,
    /// Kexp 2
    Kexp2 = 0x0A3,
    /// QUAD Settle count
    QuadSettleCount = 0x0A4,
    /// Mag. Ftest Thresh
    MagFtestThresh = 0x0A5,
    /// Mag. Ftest Thresh 2
    MagFtestThresh2 = 0x0A6,
    /// Quad cal status Tx1
    QuadCalStatusTx1 = 0x0A7,
    /// Quad cal status Tx2
    QuadCalStatusTx2 = 0x0A8,
    /// Quad cal Count
    QuadCalCount = 0x0A9,
    /// Tx Quad Full/LMT Gain
    TxQuadFullLmtGain = 0x0AA,
    /// Squarer Config
    SquarerConfig = 0x0AB,
    /// TX Quad Cal Atten
    TxQuadCalAtten = 0x0AC,
    /// Thresh Accum
    ThreshAccum = 0x0AD,
    /// Tx Quad LPF Gain
    TxQuadLpfGain = 0x0AE,
    /// TxDAC Vds I
    TxdacVdsI = 0x0B0,
    /// TxDAC Vds Q
    TxdacVdsQ = 0x0B1,
    /// TxDAC gn I
    TxdacGnI = 0x0B2,
    /// TxDAC gn Q
    TxdacGnQ = 0x0B3,
    /// TxBBF OpAmp A
    TxbbfOpampA = 0x0C0,
    /// TxBBF OpAmp B
    TxbbfOpampB = 0x0C1,
    /// Tx BBF R1
    TxBbfR1 = 0x0C2,
    /// Tx BBF R2
    TxBbfR2 = 0x0C3,
    /// Tx BBF R3
    TxBbfR3 = 0x0C4,
    /// Tx BBF R4
    TxBbfR4 = 0x0C5,
    /// Tx BBF RP
    TxBbfRp = 0x0C6,
    /// Tx BBF C1
    TxBbfC1 = 0x0C7,
    /// Tx BBF C2
    TxBbfC2 = 0x0C8,
    /// Tx BBF Cp
    TxBbfCp = 0x0C9,
    /// Tx Tune Control
    TxTuneCtrl = 0x0CA,
    /// Tx BBF R2b
    TxBbfR2b = 0x0CB,
    /// Tx BBF Tune
    TxBbfTune = 0x0CC,
    /// Config0
    Config0 = 0x0D0,
    /// Resistor
    Resistor = 0x0D1,
    /// Capacitor
    Capacitor = 0x0D2,
    /// LO CM
    LoCm = 0x0D3,
    /// TX BBF Tune Divider
    TxBbfTuneDivider = 0x0D6,
    /// TX BBF Tune Mode
    TxBbfTuneMode = 0x0D7,
    /// Rx Filter Coeff Addr
    RxFilterCoefAddr = 0x0F0,
    /// Rx Filter Coeff Data 1
    RxFilterCoefData1 = 0x0F1,
    /// Rx Filter Coeff Data 2
    RxFilterCoefData2 = 0x0F2,
    /// Rx Filter Coeff Read Data 1
    RxFilterCoefReadData1 = 0x0F3,
    /// Rx Filter Coeff Read Data 2
    RxFilterCoefReadData2 = 0x0F4,
    /// Rx Filter Config
    RxFilterConfig = 0x0F5,
    /// Rx Filter Gain
    RxFilterGain = 0x0F6,
    /// AGC Config1
    AgcConfig1 = 0x0FA,
    /// AGC config2
    AgcConfig2 = 0x0FB,
    /// AGC Config3
    AgcConfig3 = 0x0FC,
    /// Max LMT/Full Gain
    MaxLmtFullGain = 0x0FD,
    /// Peak Wait Time
    PeakWaitTime = 0x0FE,
    /// Digital Gain
    DigitalGain = 0x100,
    /// AGC Lock Level
    AgcLockLevel = 0x101,
    /// ADC noise Correction Factor
    AdcNoiseCorrectionFactor = 0x102,
    /// Gain Step Config1
    GainStepConfig1 = 0x103,
    /// ADC Small Overload Threshold
    AdcSmallOverloadThresh = 0x104,
    /// ADC Large Overload Threshold
    AdcLargeOverloadThresh = 0x105,
    /// Gain Step Config 2
    GainStepConfig2 = 0x106,
    /// Small LMT Overload Threshold
    SmallLmtOverloadThresh = 0x107,
    /// Large LMT Overload Threshold
    LargeLmtOverloadThresh = 0x108,
    /// Rx1 Manual LMT/Full Gain
    Rx1ManualLmtFullGain = 0x109,
    /// Rx1 Manual LPF gain
    Rx1ManualLpfGainOrAgcCurrentLpf = 0x10A,
    /// Rx1 Manual Digital/Forced Gain
    Rx1ManualDigitalOrForcedGain = 0x10B,
    /// Rx2 Manual LMT/Full Gain
    Rx2ManualLmtFullGain = 0x10C,
    /// Rx2 Manual LPF Gain
    Rx2ManualLpfGainOrAgcCurentLpf = 0x10D,
    /// Rx2 Manual Digital/Forced Gain
    Rx2ManualDigitalOrForcedGain = 0x10E,
    /// Config 1
    FastConfig1 = 0x110,
    /// Config 2 & Settling Delay
    FastConfig2SettlingDelay = 0x111,
    /// Energy Lost Threshold
    FastEnergyLostThresh = 0x112,
    /// Stronger Signal Threshold
    FastStrongerSignalThresh = 0x113,
    /// Low Power Threshold
    FastLowPowerThresh = 0x114,
    /// Strong Signal Freeze
    FastStrongSignalFreeze = 0x115,
    /// Final Over Range and Opt Gain
    FastFinalOverRangeAndOptGain = 0x116,
    /// Energy Detect Count
    FastEnergyDetectCount = 0x117,
    /// AGCLL Upper Limit
    FastAgcllUpperLimit = 0x118,
    /// Gain Lock Exit Count
    FastGainLockExitCount = 0x119,
    /// Initial LMT Gain Limit. LMT table splits at this value.
    FastInitialLmtGainLimit = 0x11A,
    /// Increment Time
    FastIncrementTime = 0x11B,
    /// AGC Inner Low Threshold
    AgcInnerLowThresh = 0x120,
    /// LMT Overload Counters
    LmtOverloadCounters = 0x121,
    /// ADC Overload Counters
    AdcOverloadCounters = 0x122,
    /// Gain Step1
    GainStep1 = 0x123,
    /// Gain Update Counter1
    GainUpdateCounter1 = 0x124,
    /// Gain Update Counter2
    GainUpdateCounter2 = 0x125,
    /// Digital Sat Counter
    DigitalSaturationCounter = 0x128,
    /// Outer Power Thresholds
    OuterPowerThreshs = 0x129,
    /// Gain Step 2
    GainStep2 = 0x12A,
    /// Ext LNA High Gain
    ExtLnaHighGain = 0x12C,
    /// Ext LNA Low Gain
    ExtLnaLowGain = 0x12D,
    /// Gain Table Address
    GainTableAddress = 0x130,
    /// Gain Table Write Data1
    GainTableWriteData1 = 0x131,
    /// Gain Table Write Data2
    GainTableWriteData2 = 0x132,
    /// Gain Table Write Data 3
    GainTableWriteData3 = 0x133,
    /// Gain Table Read Data 1
    GainTableReadData1 = 0x134,
    /// Gain Table Read Data 2
    GainTableReadData2 = 0x135,
    /// Gain Table Read Data 3
    GainTableReadData3 = 0x136,
    /// Gain Table Config
    GainTableConfig = 0x137,
    /// Gm Sub Table Address
    GmSubTableAddress = 0x138,
    /// Gm Sub Table Gain Word Write
    GmSubTableGainWrite = 0x139,
    /// Gm Sub Table Bias Word Write
    GmSubTableBiasWrite = 0x13A,
    /// Gm Sub Table Control Word Write
    GmSubTableCtrlWrite = 0x13B,
    /// Gm Sub Table Gain Word Read
    GmSubTableGainRead = 0x13C,
    /// Gm Sub Table Bias Word Read
    GmSubTableBiasRead = 0x13D,
    /// Gm Sub Table Control Word Read
    GmSubTableCtrlRead = 0x13E,
    /// Gm Sub Table Config
    GmSubTableConfig = 0x13F,
    /// Word Address
    WordAddress = 0x140,
    /// Gain Diff Word/Error Write
    GainDiffWorderrorWrite = 0x141,
    /// Gain Error Read
    GainErrorRead = 0x142,
    /// Config
    Config = 0x143,
    /// LNA Gain Diff Read Back
    LnaGainDiffReadBack = 0x144,
    /// Max Mixer Calibration Gain Index
    MaxMixerCalibrationGainIndex = 0x145,
    /// Temp Gain Coefficient
    TempGainCoef = 0x146,
    /// Settle Time
    SettleTime = 0x147,
    /// Measure Duration
    MeasureDuration = 0x148,
    /// Cal Temp sensor word
    CalTempSensorWord = 0x149,
    /// Measure Duration 0&1
    MeasureDuration01 = 0x150,
    /// Measure Duration 2&3
    MeasureDuration23 = 0x151,
    /// RSSI Weight 0
    RssiWeight0 = 0x152,
    /// RSSI Weight 1
    RssiWeight1 = 0x153,
    /// RSSI Weight 2
    RssiWeight2 = 0x154,
    /// RSSI Weight 3
    RssiWeight3 = 0x155,
    /// RSSI delay
    RssiDelay = 0x156,
    /// RSSI wait time
    RssiWaitTime = 0x157,
    /// RSSI Config
    RssiConfig = 0x158,
    /// ADC Measure Duration 0&1
    AdcMeasureDuration01 = 0x159,
    /// ADC Weight 0
    AdcWeight0 = 0x15A,
    /// ADC Weight 1
    AdcWeight1 = 0x15B,
    /// Dec Power Measure Duration 0
    DecPowerMeasureDuration = 0x15C,
    /// LNA Gain
    LnaGain = 0x15D,
    /// CH1 ADC Power
    Ch1AdcPower = 0x160,
    /// CH1 Rx filter Power
    Ch1RxFilterPower = 0x161,
    /// CH2 ADC Power
    Ch2AdcPower = 0x162,
    /// CH2 Rx filter Power
    Ch2RxFilterPower = 0x163,
    /// Rx Quad Cal Level
    RxQuadCalLevel = 0x168,
    /// Calibration Config 1
    CalibrationConfig1 = 0x169,
    /// Calibration config2
    CalibrationConfig2 = 0x16A,
    /// Calibration config3
    CalibrationConfig3 = 0x16B,
    /// Calib count
    CalibCount = 0x16C,
    /// Settle count
    SettleCount = 0x16D,
    /// Rx Quad gain1
    RxQuadGain1 = 0x16E,
    /// Rx Quad gain2
    RxQuadGain2 = 0x16F,
    /// Rx1 Input A Phase Corr
    Rx1InputAPhaseCorr = 0x170,
    /// Rx1 Input A Gain Corr
    Rx1InputAGainCorr = 0x171,
    /// Rx2 Input A Phase Corr
    Rx2InputAPhaseCorr = 0x172,
    /// Rx2 Input A Gain Corr
    Rx2InputAGainCorr = 0x173,
    /// Rx1 Input A Q" Offset
    Rx1InputAQOffset = 0x174,
    /// Rx1 Input A Offsets
    Rx1InputAOffsets = 0x175,
    /// Input A Offsets 1
    InputAOffsets1 = 0x176,
    /// Rx2 Input A Offsets
    Rx2InputAOffsets = 0x177,
    /// Rx2 Input A "I" Offset
    Rx2InputAIOffset = 0x178,
    /// Rx1 Input B&C Phase Corr
    Rx1InputBcPhaseCorr = 0x179,
    /// Rx1 Input B&C Gain Corr
    Rx1InputBcGainCorr = 0x17A,
    /// Rx2 Input B&C Phase Corr
    Rx2InputBcPhaseCorr = 0x17B,
    /// Rx2 Input B&C Gain Corr
    Rx2InputBcGainCorr = 0x17C,
    /// Rx1 Input B&C "Q" Offset
    Rx1InputBcQOffset = 0x17D,
    /// Rx1 Input B&C Offsets
    Rx1InputBcOffsets = 0x17E,
    /// Input B&C Offsets 1
    InputBcOffsets1 = 0x17F,
    /// Rx2 Input B&C Offsets
    Rx2InputBcOffsets = 0x180,
    /// Rx2 Input B&C "I" Offset
    Rx2InputBcIOffset = 0x181,
    /// Force Bits
    ForceBits = 0x182,
    /// Wait Count
    WaitCount = 0x185,
    /// RF DC Offset Count
    RfDcOffsetCount = 0x186,
    /// RF DC Offset Config1
    RfDcOffsetConfig1 = 0x187,
    /// RF DC Offset Attenuation
    RfDcOffsetAtten = 0x188,
    /// Invert Bits
    InvertBits = 0x189,
    /// DC Offset Config2
    DcOffsetConfig2 = 0x18B,
    /// RF Cal Gain Index
    RfCalGainIndex = 0x18C,
    /// SOI Threshold
    SoiThresh = 0x18D,
    /// BB DC Offset Shift
    BbDcOffsetShift = 0x190,
    /// BB DC Offset Fast Settle Shift
    BbDcOffsetFastSettleShift = 0x191,
    /// BB Fast Settle Dur
    BbFastSettleDur = 0x192,
    /// BB DC Offset Count
    BbDcOffsetCount = 0x193,
    /// BB DC Offset Attenuation
    BbDcOffsetAtten = 0x194,
    /// RX1 BB DC word I MSB
    Rx1BbDcWordIMsb = 0x19A,
    /// RX1 BB DC word I LSB
    Rx1BbDcWordILsb = 0x19B,
    /// RX1 BB DC word Q MSB
    Rx1BbDcWordQMsb = 0x19C,
    /// RX1 BB DC word Q LSB
    Rx1BbDcWordQLsb = 0x19D,
    /// RX2 BB DC word I MSB
    Rx2BbDcWordIMsb = 0x19E,
    /// RX2 BB DC word I LSB
    Rx2BbDcWordILsb = 0x19F,
    /// RX2 BB DC word Q MSB
    Rx2BbDcWordQMsb = 0x1A0,
    /// RX2 BB DC word Q LSB
    Rx2BbDcWordQLsb = 0x1A1,
    /// BB Track corr word I MSB
    BbTrackCorrWordIMsb = 0x1A2,
    /// BB Track corr word I LSB
    BbTrackCorrWordILsb = 0x1A3,
    /// BB Track corr word Q MSB
    BbTrackCorrWordQMsb = 0x1A4,
    /// BB Track corr word Q LSB
    BbTrackCorrWordQLsb = 0x1A5,
    /// Rx1 RSSI Symbol
    Rx1RssiSymbol = 0x1A7,
    /// Rx1 RSSI preamble
    Rx1RssiPreamble = 0x1A8,
    /// Rx2 RSSI symbol
    Rx2RssiSymbol = 0x1A9,
    /// Rx2 RSSI preamble
    Rx2RssiPreamble = 0x1AA,
    /// Symbol LSB
    SymbolLsb = 0x1AB,
    /// Preamble LSB
    PreambleLsb = 0x1AC,
    /// Rx Path Gain
    RxPathGainMsb = 0x1AD,
    /// Rx Path Gain
    RxPathGainLsb = 0x1AE,
    /// Rx Diff LNA Force
    RxDiffLnaForce = 0x1B0,
    /// Rx LNA Bias Coarse
    RxLnaBiasCoarse = 0x1B1,
    /// Rx LNA Bias Fine 0
    RxLnaBiasFine0 = 0x1B2,
    /// Rx LNA Bias Fine 1
    RxLnaBiasFine1 = 0x1B3,
    /// Rx Mix Gm Config
    RxMixGmConfig = 0x1C0,
    /// Rx1 Mix Gm Force
    Rx1MixGmForce = 0x1C1,
    /// Rx1 Mix Gm Bias (Force)
    Rx1MixGmBiasForce = 0x1C2,
    /// Rx2 Mix Gm Force
    Rx2MixGmForce = 0x1C3,
    /// Rx2 Mix Gm Bias (Force)
    Rx2MixGmBiasForce = 0x1C4,
    /// Input A MSBs
    InputAMsbs = 0x1C8,
    /// Input A RX1 I
    InputARx1I = 0x1C9,
    /// Input A RX1 Q
    InputARx1Q = 0x1CA,
    /// Input A RX2 I
    InputARx2I = 0x1CB,
    /// Input A RX2 Q
    InputARx2Q = 0x1CC,
    /// Inputs B&C RX1 I
    InputsBcRx1I = 0x1CD,
    /// Band1 RX1 Q
    Band1Rx1Q = 0x1CE,
    /// Inputs B&C RX2 I
    InputsBcRx2I = 0x1CF,
    /// Inputs B&C RX2 Q
    InputsBcRx2Q = 0x1D0,
    /// Inputs B&C MSBs
    InputsBcMsbs = 0x1D1,
    /// Force OS DAC
    ForceOsDac = 0x1D2,
    /// Rx Mix LO CM
    RxMixLoCm = 0x1D5,
    /// Rx CGB Seg Enable
    RxCgbSegEnable = 0x1D6,
    /// Rx Mix Input/Bias
    RxMixInputbias = 0x1D7,
    /// Rx TIA Config
    RxTiaConfig = 0x1DB,
    /// TIA1 C LSB
    Tia1CLsb = 0x1DC,
    /// TIA1 C MSB
    Tia1CMsb = 0x1DD,
    /// TIA2 C LSB
    Tia2CLsb = 0x1DE,
    /// TIA2 C MSB
    Tia2CMsb = 0x1DF,
    /// Rx1 BBF R1A
    Rx1BbfR1a = 0x1E0,
    /// Rx2 BBF R1A
    Rx2BbfR1a = 0x1E1,
    /// Rx1 Tune Control
    Rx1TuneCtrl = 0x1E2,
    /// Rx2 Tune Control
    Rx2TuneCtrl = 0x1E3,
    /// Rx1 BBF R5
    Rx1BbfR5 = 0x1E4,
    /// Rx2 BBF R5
    Rx2BbfR5 = 0x1E5,
    /// Rx BBF R2346
    RxBbfR2346 = 0x1E6,
    /// Rx BBF C1 MSB
    RxBbfC1Msb = 0x1E7,
    /// Rx BBF C1 LSB
    RxBbfC1Lsb = 0x1E8,
    /// Rx BBF C2 MSB
    RxBbfC2Msb = 0x1E9,
    /// Rx BBF C2 LSB
    RxBbfC2Lsb = 0x1EA,
    /// Rx BBF C3 MSB
    RxBbfC3Msb = 0x1EB,
    /// Rx BBF C3 LSB
    RxBbfC3Lsb = 0x1EC,
    /// Rx BBF CC1 Ctr
    RxBbfCc1Ctr = 0x1ED,
    /// Rx BBF Pow Rz Byte0
    RxBbfPowRzByte0 = 0x1EE,
    /// Rx BBF CC2 Ctr
    RxBbfCc2Ctr = 0x1EF,
    /// Rx BBF Pow Rz Byte1
    RxBbfPowRzByte1 = 0x1F0,
    /// Rx BBF CC3 Ctr
    RxBbfCc3Ctr = 0x1F1,
    /// Rx BBF R5 Tune
    RxBbfR5Tune = 0x1F2,
    /// Rx BBF Tune
    RxBbfTune = 0x1F3,
    /// Rx1 BBF Man Gain
    Rx1BbfManGain = 0x1F4,
    /// Rx2 BBF Man Gain
    Rx2BbfManGain = 0x1F5,
    /// RX BBF Tune Divide
    RxBbfTuneDivide = 0x1F8,
    /// RX BBF Tune Config
    RxBbfTuneConfig = 0x1F9,
    /// Pole gain
    PoleGain = 0x1FA,
    /// Rx BBBW MHz
    RxBbbwMhz = 0x1FB,
    /// Rx BBBW kHz
    RxBbbwKhz = 0x1FC,
    /// ADC Setup Byte 0
    AdcSetup0 = 0x200,
    /// FB DAC Clk Delay1
    FbDacClkDelay1 = 0x201,
    /// FB DAC Clk Delay2
    FbDacClkDelay2 = 0x202,
    /// Flash Sample Clk Delay 3p
    FlashSampleClkDelay3p = 0x203,
    /// Flash Sample Clk Delay 3n
    FlashSampleClkDelay3n = 0x204,
    /// Test MUX 2i
    TestMux2i = 0x205,
    /// Test MUX 2q
    TestMux2q = 0x206,
    /// Integrator 1 Resistance
    Integrator1Resistance = 0x207,
    /// Integrator 1 Capacitance
    Integrator1Capacitance = 0x208,
    /// Integrator 23 Resistance
    Integrator23Resistance = 0x209,
    /// Integrator 2 Resistance
    Integrator2Resistance = 0x20A,
    /// Integrator 2 Capacitance
    Integrator2Capacitance = 0x20B,
    /// Integrator 3 Resistance
    Integrator3Resistance = 0x20C,
    /// Integrator 3 Capacitance
    Integrator3Capacitance = 0x20D,
    /// Integrator Amp Cc
    IntegratorAmpCc = 0x20E,
    /// Int 1 FB DAC NMOS Current Source
    Int1FbDacNmosCurrentSource = 0x20F,
    /// Int 1 FB DAC NMOS Casoade Bias Current
    Int1FbDacNmosCasoadeBiasCurrent = 0x210,
    /// Int 1 FB DAC PMOS Current Source
    Int1FbDacPmosCurrentSource = 0x211,
    /// Int 2 FB DAC NMOS Current Source
    Int2FbDacNmosCurrentSource = 0x212,
    /// Int 2 FB DAC NMOS Cascode Bias Current
    Int2FbDacNmosCascodeBiasCurrent = 0x213,
    /// Int 2 FB DAC PMOS Current Source
    Int2FbDacPmosCurrentSource = 0x214,
    /// Int 3 FB DAC NMOS Current Source
    Int3FbDacNmosCurrentSource = 0x215,
    /// Int 3 FB DAC NMOS Cascode Bias Current
    Int3FbDacNmosCascodeBiasCurrent = 0x216,
    /// Int 3 FB DAC PMOS Current Source
    Int3FbDacPmosCurrentSource = 0x217,
    /// FB DAC Bias Current
    FbDacBiasCurrent = 0x218,
    /// Int 1 1st Stage Current
    Int11stStageCurrent = 0x219,
    /// Int 1 1st Stage Cascode Current
    Int11stStageCascodeCurrent = 0x21A,
    /// Int 1 2nd Stage Current
    Int12ndStageCurrent = 0x21B,
    /// Integrator 2 1st Stage Current
    Integrator21stStageCurrent = 0x21C,
    /// Int 2 1st Stage Cascode Current
    Int21stStageCascodeCurrent = 0x21D,
    /// Int 2 2nd Stage Current
    Int22ndStageCurrent = 0x21E,
    /// Int 3 1st Stage Current
    Int31stStageCurrent = 0x21F,
    /// Int 3 1st Stage Cascode Current
    Int31stStageCascodeCurrent = 0x220,
    /// Int 3 2nd Stage Current
    Int32ndStageCurrent = 0x221,
    /// Flash Bias Current
    FlashBiasCurrent = 0x222,
    /// Flash Ladder Bias
    FlashLadderBias = 0x223,
    /// Flash Ladder Cascode Current
    FlashLadderCascodeCurrent = 0x224,
    /// Flash Ladder Bias2
    FlashLadderBias2 = 0x225,
    /// Reset
    Reset = 0x226,
    /// ADC Setup Byte 39
    AdcSetup39 = 0x227,
    /// RX PFD Config
    RxPfdConfig = 0x230,
    /// RX Integer Byte 0
    RxIntegerByte0 = 0x231,
    /// RX Integer Byte 1
    RxIntegerByte1 = 0x232,
    /// RX Fractional Byte 0
    RxFractByte0 = 0x233,
    /// RX Fractional Byte 1
    RxFractByte1 = 0x234,
    /// RX Fractional Byte 2
    RxFractByte2 = 0x235,
    /// RX Force ALC
    RxForceAlc = 0x236,
    /// RX Force VCO Tune 0
    RxForceVcoTune0 = 0x237,
    /// RX Force VCO Tune 1
    RxForceVcoTune1 = 0x238,
    /// RX ALC/Varactor
    RxAlcVaractor = 0x239,
    /// RX VCO Output
    RxVcoOutput = 0x23A,
    /// RX CP Current
    RxCpCurrent = 0x23B,
    /// RX CP Offset
    RxCpOffset = 0x23C,
    /// RX CP Config
    RxCpConfig = 0x23D,
    /// RX Loop Filter 1
    RxLoopFilter1 = 0x23E,
    /// RX Loop Filter 2
    RxLoopFilter2 = 0x23F,
    /// RX Loop Filter 3
    RxLoopFilter3 = 0x240,
    /// RX Dither/CP Cal
    RxDithercpCal = 0x241,
    /// RX VCO Bias 1
    RxVcoBias1 = 0x242,
    /// RX Cal Status
    RxCalStatus = 0x244,
    /// RX VCO Cal Ref
    RxVcoCalRef = 0x245,
    /// RX VCO Pd Overrides
    RxVcoPdOverrides = 0x246,
    /// RX CP Over Range/VCO Lock
    RxCpOverrangeVcoLock = 0x247,
    /// RX VCO LDO
    RxVcoLdo = 0x248,
    /// RX VCO Cal
    RxVcoCal = 0x249,
    /// RX Lock Detect Config
    RxLockDetectConfig = 0x24A,
    /// RX CP Level Detect
    RxCpLevelDetect = 0x24B,
    /// RX DSM Setup 0
    RxDsmSetup0 = 0x24C,
    /// RX DSM Setup 1
    RxDsmSetup1 = 0x24D,
    /// RX Correction Word0
    RxCorrectionWord0 = 0x24E,
    /// RX Correction Word1
    RxCorrectionWord1 = 0x24F,
    /// RX VCO Varactor Control 0
    RxVcoVaractorCtrl0 = 0x250,
    /// RX VCO Varactor Control 1
    RxVcoVaractorCtrl1 = 0x251,
    /// Rx Fast Lock Setup
    RxFastLockSetup = 0x25A,
    /// Rx Fast Lock Setup Init Delay
    RxFastLockSetupInitDelay = 0x25B,
    /// Rx Fast Lock Program Addr
    RxFastLockProgramAddr = 0x25C,
    /// Rx Fast Lock Program Data
    RxFastLockProgramData = 0x25D,
    /// Rx Fast Lock Program Read
    RxFastLockProgramRead = 0x25E,
    /// Rx Fast Lock Program Control
    RxFastLockProgramCtrl = 0x25F,
    /// Rx LO Gen Power Mode
    RxLoGenPowerMode = 0x261,
    /// TX PFD Config
    TxPfdConfig = 0x270,
    /// TX Integer Byte 0
    TxIntegerByte0 = 0x271,
    /// TX Integer Byte 1
    TxIntegerByte1 = 0x272,
    /// TX Fractional Byte 0
    TxFractByte0 = 0x273,
    /// TX Fractional Byte 1
    TxFractByte1 = 0x274,
    /// TX Fractional Byte 2
    TxFractByte2 = 0x275,
    /// TX Force ALC
    TxForceAlc = 0x276,
    /// TX Force VCO Tune 0
    TxForceVcoTune0 = 0x277,
    /// TX Force VCO Tune 1
    TxForceVcoTune1 = 0x278,
    /// TX ALC/Varactor
    TxAlcVaractor = 0x279,
    /// TX VCO Output
    TxVcoOutput = 0x27A,
    /// TX CP Current
    TxCpCurrent = 0x27B,
    /// TX CP Offset
    TxCpOffset = 0x27C,
    /// TX CP Config
    TxCpConfig = 0x27D,
    /// TX Loop Filter 1
    TxLoopFilter1 = 0x27E,
    /// TX Loop Filter 2
    TxLoopFilter2 = 0x27F,
    /// TX Loop Filter 3
    TxLoopFilter3 = 0x280,
    /// TX Dither/CP Cal
    TxDithercpCal = 0x281,
    /// TX VCO Bias 1
    TxVcoBias1 = 0x282,
    /// TX VCO Bias 2
    TxVcoBias2 = 0x283,
    /// TX Cal Status
    TxCalStatus = 0x284,
    /// TX VCO Cal Ref
    TxVcoCalRef = 0x285,
    /// TX VCO Pd Overrides
    TxVcoPdOverrides = 0x286,
    /// TX CP Over Range/VCO Lock
    TxCpOverrangeVcoLock = 0x287,
    /// TX VCO LDO
    TxVcoLdo = 0x288,
    /// TX VCO Cal
    TxVcoCal = 0x289,
    /// TX Lock Detect Config
    TxLockDetectConfig = 0x28A,
    /// TX CP Level Detect
    TxCpLevelDetect = 0x28B,
    /// TX DSM Setup 0
    TxDsmSetup0 = 0x28C,
    /// TX DSM Setup 1
    TxDsmSetup1 = 0x28D,
    /// TX Correction Word0
    TxCorrectionWord0 = 0x28E,
    /// TX Correction Word1
    TxCorrectionWord1 = 0x28F,
    /// TX VCO Varactor Control 0
    TxVcoVaractorCtrl0 = 0x290,
    /// TX VCO Varactor Control 1
    TxVcoVaractorCtrl1 = 0x291,
    /// DCXO Coarse Tune
    DcxoCoarseTune = 0x292,
    /// DCXO Fine Tune2
    DcxoFineTuneHigh = 0x293,
    /// DCXO Fine Tune1
    DcxoFineTuneLow = 0x294,
    /// DCXO Config
    DcxoConfig = 0x295,
    /// DCXO Tempco Write
    DcxoTempcoWrite = 0x296,
    /// DCXO Tempco Read
    DcxoTempcoRead = 0x297,
    /// DCXO Tempco Addr
    DcxoTempcoAddr = 0x298,
    /// Delta T Read
    DeltaTRead = 0x299,
    /// Tx Fast Lock Setup
    TxFastLockSetup = 0x29A,
    /// Tx Fast Lock Setup Init Delay
    TxFastLockSetupInitDelay = 0x29B,
    /// Tx Fast Lock Program Addr
    TxFastLockProgramAddr = 0x29C,
    /// Tx Fast Lock Program Data
    TxFastLockProgramData = 0x29D,
    /// Tx Fast Lock Program Read
    TxFastLockProgramRead = 0x29E,
    /// Tx Fast Lock Program Ctrl
    TxFastLockProgramCtrl = 0x29F,
    /// Tx LO Gen Power Mode
    TxLoGenPowerMode = 0x2A1,
    /// Bandgap Config0
    BandgapConfig0 = 0x2A6,
    /// Bandgap Config1
    BandgapConfig1 = 0x2A8,
    /// Ref Divide Config 1
    RefDivideConfig1 = 0x2AB,
    /// Ref Divide Config 2
    RefDivideConfig2 = 0x2AC,
    /// Gain Rx1
    GainRx1 = 0x2B0,
    /// LPF Gain Rx1
    LpfGainRx1 = 0x2B1,
    /// Dig gain Rx1
    DigGainRx1 = 0x2B2,
    /// Fast Attack State
    FastAttackState = 0x2B3,
    /// Slow Loop State
    SlowLoopState = 0x2B4,
    /// Gain Rx2
    GainRx2 = 0x2B5,
    /// LPF Gain Rx2
    LpfGainRx2 = 0x2B6,
    /// Dig Gain Rx2
    DigGainRx2 = 0x2B7,
    /// Ovrg Sigs Rx1
    OvrgSigsRx1 = 0x2B8,
    /// Ovrg Sigs Rx2
    OvrgSigsRx2 = 0x2B9,
    /// Control
    Ctrl = 0x3DF,
    /// BIST Config
    BistConfig = 0x3F4,
    /// BIST Config 2
    BistConfig2 = 0x3F5,
    /// BIST and Data Port Test Config
    BistAndDataPortTestConfig = 0x3F6,
    /// DAC Test 0
    DacTest0 = 0x3FC,
    /// DAC Test 1
    DacTest1 = 0x3FD,
    /// DAC Test 2
    DacTest2 = 0x3FE,
}

impl Register {
    /// Get regsiter address as raw [u10].
    pub const fn as_u10(&self) -> u10 {
        u10::new(*self as u16)
    }
}

pub mod spi_conf {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(7, rw)]
        soft_reset: bool,
        /// SDO Active (3-Wire SPI)
        #[bit(6, rw)]
        wire3_spi: bool,
        #[bit(5, rw)]
        lsb_first: bool,
        /// Mirror of bit 5
        #[bit(2, rw)]
        lsb_first_mirror: bool,
        /// Mirror of bit 6
        #[bit(1, rw)]
        wire3_spi_mirror: bool,
        /// Mirror of bit 7
        #[bit(0, rw)]
        soft_reset_mirror: bool,
    }
}

pub mod multichip_sync_and_tx_mon_ctrl {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(6, rw)]
        tx2_monitor_enable: bool,
        #[bit(5, rw)]
        tx1_monitor_enable: bool,
        #[bit(3, rw)]
        mcs_rf_enable: bool,
        #[bit(2, rw)]
        mcs_bbpll_enable: bool,
        #[bit(1, rw)]
        mcs_digital_clk_enable: bool,
        #[bit(0, rw)]
        mcs_bb_enable: bool,
    }
}

pub mod tx_enable_filter_ctrl {
    #[bitbybit::bitenum(u2, exhaustive = false)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum Thb3Interpolation {
        Mult1NoFiltering = 0b00,
        Mult2HalfBand = 0b01,
        Mult3Filter = 0b10,
    }

    impl Thb3Interpolation {
        #[inline]
        pub const fn multiplier(&self) -> u32 {
            match self {
                Self::Mult1NoFiltering => 1,
                Self::Mult2HalfBand => 2,
                Self::Mult3Filter => 3,
            }
        }

        pub const fn from_raw_multiplier(div: u8) -> Option<Self> {
            match div {
                1 => Some(Self::Mult1NoFiltering),
                2 => Some(Self::Mult2HalfBand),
                3 => Some(Self::Mult3Filter),
                _ => None,
            }
        }
    }

    #[bitbybit::bitenum(u2, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum TxFirInterpolation {
        Mult1BypassFilter = 0b00,
        Mult1EnableFilter = 0b01,
        Mult2EnableFilter = 0b10,
        Mult4EnableFilter = 0b11,
    }

    impl TxFirInterpolation {
        #[inline]
        pub const fn multiplier(&self) -> u32 {
            match self {
                Self::Mult1BypassFilter => 1,
                Self::Mult1EnableFilter => 1,
                Self::Mult2EnableFilter => 2,
                Self::Mult4EnableFilter => 4,
            }
        }
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(7, rw)]
        enable_transmitter_2: bool,
        #[bit(6, rw)]
        enable_transmitter_1: bool,

        #[bits(4..=5, rw)]
        thb3: Option<Thb3Interpolation>,

        /// THB2 Enable
        #[bit(3, rw)]
        thb2: bool,

        /// THB1 Enable
        #[bit(2, rw)]
        thb1: bool,

        #[bits(0..=1, rw)]
        tx_fir: TxFirInterpolation,
    }
}

pub mod rx_enable_filter_ctrl {
    #[bitbybit::bitenum(u2, exhaustive = false)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum Rhb3Decimation {
        Div1NoFiltering = 0b00,
        Div2HalfBand = 0b01,
        Div3Filter = 0b10,
    }

    impl Rhb3Decimation {
        #[inline]
        pub const fn divisor(&self) -> u32 {
            match self {
                Rhb3Decimation::Div1NoFiltering => 1,
                Rhb3Decimation::Div2HalfBand => 2,
                Rhb3Decimation::Div3Filter => 3,
            }
        }

        pub const fn from_raw_divisor(div: u8) -> Option<Self> {
            match div {
                1 => Some(Self::Div1NoFiltering),
                2 => Some(Self::Div2HalfBand),
                3 => Some(Self::Div3Filter),
                _ => None,
            }
        }
    }

    #[bitbybit::bitenum(u2, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum RxFirDecimation {
        Div1BypassFilter = 0b00,
        Div1EnableFilter = 0b01,
        Div2EnableFilter = 0b10,
        Div4EnableFilter = 0b11,
    }

    impl RxFirDecimation {
        #[inline]
        pub const fn divisor(&self) -> u32 {
            match self {
                RxFirDecimation::Div1BypassFilter => 1,
                RxFirDecimation::Div1EnableFilter => 1,
                RxFirDecimation::Div2EnableFilter => 2,
                RxFirDecimation::Div4EnableFilter => 4,
            }
        }
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(7, rw)]
        enable_receiver_2: bool,
        #[bit(6, rw)]
        enable_receiver_1: bool,

        #[bits(4..=5, rw)]
        rhb3: Option<Rhb3Decimation>,

        /// RHB2 Enable
        #[bit(3, rw)]
        rhb2: bool,

        /// RHB1 Enable
        #[bit(2, rw)]
        rhb1: bool,

        #[bits(0..=1, rw)]
        rx_fir: RxFirDecimation,
    }
}

pub mod input_select {
    /// Configure the TX RF output channels.
    #[bitbybit::bitenum(u1, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum TxRfPathConfig {
        PortA,
        PortB,
    }

    /// Configure the RX RF input channels.
    #[bitbybit::bitenum(u6, exhaustive = false)]
    #[derive(Debug, PartialEq, Eq)]
    #[allow(non_camel_case_types)]
    pub enum RxRfPathConfig {
        /// Single-ended: enables the A negative input on RX1 and RX2.
        SingleEnded_Rx1A_N_Rx2A_N = 0b000001,
        /// Single-ended: enables the A positive input on RX1 and RX2.
        SingleEnded_Rx1A_P_Rx2A_P = 0b000010,
        /// Single-ended: enables the B negative input on RX1 and RX2.
        SingleEnded_Rx1B_N_Rx2B_N = 0b000100,
        /// Single-ended: enables the B positive input on RX1 and RX2.
        SingleEnded_Rx1B_P_Rx2B_P = 0b001000,
        /// Single-ended: enables the C negative input on RX1 and RX2.
        SingleEnded_Rx1C_N_Rx2C_N = 0b010000,
        /// Single-ended: enables the C positive input on RX1 and RX2.
        SingleEnded_Rx1C_P_Rx2C_P = 0b100000,

        /// Differential: enables both A inputs as a differential pair on RX1 and RX2.
        Diff_Rx1A_NP_Rx2A_NP = 0b000011,
        /// Differential: enables both B inputs as a differential pair on RX1 and RX2.
        Diff_Rx1B_NP_Rx2B_NP = 0b001100,
        /// Differential: enables both C inputs as a differential pair on RX1 and RX2.
        Diff_Rx1C_NP_Rx2C_NP = 0b110000,
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(6, rw)]
        tx_config: TxRfPathConfig,
        #[bits(0..=5, rw)]
        rx_config: Option<RxRfPathConfig>,
    }
}

pub mod rfpll_dividers {
    pub use arbitrary_int::u4;

    use crate::clocks::PllVcoDivider;

    #[bitbybit::bitenum(u4, exhaustive = false)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum PllVcoDividerBits {
        Div2 = 0,
        Div4 = 1,
        Div8 = 2,
        Div16 = 3,
        Div32 = 4,
        Div64 = 5,
        Div128 = 6,
        External = 7,
    }

    impl From<PllVcoDivider> for PllVcoDividerBits {
        fn from(value: PllVcoDivider) -> Self {
            value.as_reg_bits()
        }
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// TX VCO Divider<3:0>
        #[bits(4..=7, rw)]
        tx_vco_divider: Option<PllVcoDividerBits>,
        /// RX VCO Divider<3:0>
        #[bits(0..=3, rw)]
        rx_vco_divider: Option<PllVcoDividerBits>,
    }
}

pub mod rx_clock_data_delay {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// DATA_CLK Delay<3:0>
        #[bits(4..=7, rw)]
        data_clk_delay: u4,
        /// Rx Data Delay<3:0>
        #[bits(0..=3, rw)]
        rx_data_delay: u4,
    }
}

pub mod tx_clock_data_delay {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// FB_CLK Delay<3:0>
        #[bits(4..=7, rw)]
        fb_clk_delay: u4,
        /// Tx Data Delay<3:0>
        #[bits(0..=3, rw)]
        tx_data_delay: u4,
    }
}

pub mod clock_enable {

    #[bitbybit::bitenum(u1, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum ExternalClockConfig {
        /// External oscillator connected to the REF_CLK_IN pin.
        Oscillator = 1,
        /// External crystal in combination with internal digital programmable capacitor.
        Dcxo = 0,
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(4, rw)]
        xo_bypass: ExternalClockConfig,
        #[bit(2, rw)]
        digital_power_up: bool,
        #[bit(1, rw)]
        clock_enable_dflt: bool,
        #[bit(0, rw)]
        bbpll_enable: bool,
    }
}

pub mod clk_bb_pll {
    pub use arbitrary_int::u3;

    #[bitbybit::bitenum(u3, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum ClkOutMode {
        XtalnN = 0b000,
        AdcClkDiv2 = 0b001,
        AdcClkDiv3 = 0b010,
        AdcClkDiv4 = 0b011,
        AdcClkDiv8 = 0b100,
        AdcClkDiv16 = 0b101,
        AdcClkDiv32 = 0b110,
        AdcClkDiv64 = 0b111,
    }

    #[bitbybit::bitenum(u3, exhaustive = false)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum AdcDivisor {
        Div2 = 1,
        Div4 = 2,
        Div8 = 3,
        Div16 = 4,
        Div32 = 5,
        Div64 = 6,
    }

    impl AdcDivisor {
        /// Divisor applied to the BB PLL clock to obtain the ADC master clock.
        pub const fn divisor(&self) -> u32 {
            match self {
                AdcDivisor::Div2 => 2,
                AdcDivisor::Div4 => 4,
                AdcDivisor::Div8 => 8,
                AdcDivisor::Div16 => 16,
                AdcDivisor::Div32 => 32,
                AdcDivisor::Div64 => 64,
            }
        }

        pub const fn next_smaller(&self) -> Self {
            match self {
                AdcDivisor::Div2 => AdcDivisor::Div2,
                AdcDivisor::Div4 => AdcDivisor::Div2,
                AdcDivisor::Div8 => AdcDivisor::Div4,
                AdcDivisor::Div16 => AdcDivisor::Div8,
                AdcDivisor::Div32 => AdcDivisor::Div16,
                AdcDivisor::Div64 => AdcDivisor::Div32,
            }
        }
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(5..=7, rw)]
        clk_out_select: ClkOutMode,
        #[bit(4, rw)]
        clkout_enable: bool,
        #[bit(3, rw)]
        dac_clk_div2: bool,
        #[bits(0..=2, rw)]
        bb_pll_div: Option<AdcDivisor>,
    }
}

pub mod start_temp_reading {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(0, rw)]
        pub clear_to_start_temp_reading: bool,
    }
}

pub mod temp_sense2 {
    pub use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(1..=7, rw)]
        measurement_time_interval: u7,
        #[bit(0, rw)]
        temp_sense_periodic_enable: bool,
    }
}

pub mod temp_sensor_config {
    pub use arbitrary_int::u3;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(0..=2, rw)]
        decimation: u3,
    }
}

pub mod parallel_port_conf_1 {
    #[bitbybit::bitfield(
        u8,
        default = 0xC0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// PP Tx Swap IQ
        #[bit(7, rw)]
        pp_tx_swap_iq: bool,
        /// PP Rx Swap IQ
        #[bit(6, rw)]
        pp_rx_swap_iq: bool,
        /// Tx Channel swap
        #[bit(5, rw)]
        tx_channel_swap: bool,
        /// Rx Channel swap
        #[bit(4, rw)]
        rx_channel_swap: bool,
        /// Rx Frame Pulse Mode
        #[bit(3, rw)]
        rx_frame_pulse_mode: bool,
        /// 2R2T Timing
        #[bit(2, rw)]
        r2t2_timing: bool,
        /// Invert data bus
        #[bit(1, rw)]
        invert_data_bus: bool,
        /// Invert DATA CLK
        #[bit(0, rw)]
        invert_data_clk: bool,
    }
}

pub mod parallel_port_conf_2 {
    use arbitrary_int::u2;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// FDD Alt Word Order
        #[bit(7, rw)]
        fdd_alt_word_order: bool,
        /// Invert Rx1
        #[bit(6, rw)]
        invert_rx1: bool,
        /// Invert Rx2
        #[bit(5, rw)]
        invert_rx2: bool,
        /// Invert Tx1
        #[bit(4, rw)]
        invert_tx1: bool,
        /// Invert Tx2
        #[bit(3, rw)]
        invert_tx2: bool,
        /// Invert Rx Frame
        #[bit(2, rw)]
        invert_rx_frame: bool,
        /// Delay Rx Data\[1:0\]
        #[bits(0..=1, rw)]
        delay_rx_data: u2,
    }
}

pub mod parallel_port_conf_3 {
    #[bitbybit::bitenum(u1, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum DataRate {
        Single = 1,
        /// DDR, both edges of DATA_CLK are used.
        Double = 0,
    }

    #[bitbybit::bitenum(u1, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum Duplex {
        /// Data only flows into both directions.
        Full = 0,
        /// Data only flows into one direction at a time.
        Half = 1,
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    #[derive(PartialEq, Eq)]
    pub struct Register {
        /// FDD Rx Rate = 2*Tx Rate. Can only be set in full duplex mode.
        #[bit(7, rw)]
        fdd_rx_rate_2tx_rate: bool,
        /// Swap Ports 0 and 1. Must be clear for LVDS mode.
        #[bit(6, rw)]
        swap_ports: bool,
        /// Single Data Rate
        #[bit(5, rw)]
        data_rate: DataRate,
        /// LVDS Mode
        #[bit(4, rw)]
        lvds_mode: bool,
        /// Half Duplex Mode
        #[bit(3, rw)]
        duplex_mode: Duplex,
        /// Single Port Mode
        #[bit(2, rw)]
        single_port_mode: bool,
        /// Full Port, used for dual port full duplex mode (CMOS)
        ///
        /// Used only in full duplex mode and dual port mode.
        /// Setting this bit forces the receivers to be on one port
        /// and the transmitters to be on the on the other port. Clearing the
        /// bit mixes receivers and transmitters on each port.
        #[bit(1, rw)]
        full_port: bool,
        /// Full Duplex Swap Bits.
        ///
        /// This bit toggles between the bits used for receive data and those
        /// used for transmit data with one exception. If the FDD Alt Word
        /// Order bit (0x011\[D7\]) of parallel port configuration 2 is set, then the effect is to
        /// swap the most significant 6 bits with the least significant 6 bits. It is not always
        /// valid to set this bit.
        #[bit(0, rw)]
        full_duplex_swap_bits: bool,
    }
}

pub mod ensm_mode {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// FDD Mode
        #[bit(0, rw)]
        fdd_mode: bool,
    }
}

pub mod ensm_config_1 {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Enable Rx Data Port for Cal
        #[bit(7, rw)]
        enable_rx_data_port_for_cal: bool,
        /// Force Rx On
        #[bit(6, rw)]
        force_rx_on: bool,
        /// Force Tx On
        #[bit(5, rw)]
        force_tx_on: bool,
        /// Enable ENSM Pin Control
        #[bit(4, rw)]
        enable_ensm_pin_ctrl: bool,
        /// Level Mode
        #[bit(3, rw)]
        level_mode: bool,
        /// Force Alert State
        #[bit(2, rw)]
        force_alert_state: bool,
        /// Auto Gain Lock
        #[bit(1, rw)]
        auto_gain_lock: bool,
        /// To Alert
        #[bit(0, rw)]
        to_alert: bool,
    }
}

pub mod ensm_config_2 {

    #[bitbybit::bitenum(u1, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum TxNRxSpiCtrl {
        Rx = 0,
        Tx = 1,
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// FDD External Control Enable
        #[bit(7, rw)]
        fdd_external_ctrl_enable: bool,
        /// Power Down Rx Synth
        #[bit(6, rw)]
        power_down_rx_synth: bool,
        /// Power Down Tx Synth
        #[bit(5, rw)]
        power_down_tx_synth: bool,
        /// TXNRX SPI Control
        #[bit(4, rw)]
        txnrx_spi_ctrl: TxNRxSpiCtrl,
        /// Synth Enable Pin Control Mode
        #[bit(3, rw)]
        synth_enable_pin_ctrl_mode: bool,
        /// Dual Synth Mode
        #[bit(2, rw)]
        dual_synth_mode: bool,
        /// Rx Synth Ready Mask
        #[bit(1, rw)]
        rx_synth_ready_mask: bool,
        /// Tx Synth Ready Mask
        #[bit(0, rw)]
        tx_synth_ready_mask: bool,
    }
}

pub mod calibration_ctrl {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Rx BB Tune
        #[bit(7, rw)]
        rx_bb_tune_cal: bool,
        /// Tx BB Tune
        #[bit(6, rw)]
        tx_bb_tune_cal: bool,
        /// Rx Quad Cal
        #[bit(5, rw)]
        rx_quad_cal: bool,
        /// Tx Quad Cal
        #[bit(4, rw)]
        tx_quad_cal: bool,
        /// Rx Gain Step Cal
        #[bit(3, rw)]
        rx_gain_step_cal: bool,
        /// TXMON Cal
        #[bit(2, rw)]
        txmon_cal: bool,
        /// DC Cal RF Start
        #[bit(1, rw)]
        rfdc_cal: bool,
        /// DC cal BB Start
        #[bit(0, rw)]
        bbdc_cal: bool,
    }
}

pub mod state {
    pub use arbitrary_int::u4;

    #[bitbybit::bitenum(u4, exhaustive = false)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum EnsmState {
        SleepWait = 0x0,
        Alert = 0x5,
        Tx = 0x6,
        TxFlush = 0x7,
        Rx = 0x8,
        RxFlush = 0x9,
        Fdd = 0xA,
        FddFlush = 0xB,
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Calibration Sequence State<3:0>
        #[bits(4..=7, r)]
        calibration_sequence_state: u4,
        /// ENSM State<3:0>
        #[bits(0..=3, r)]
        ensm_state: Option<EnsmState>,
    }
}

pub mod auxdac_2_word {
    /// AuxDAC 2 Word<9:2>
    pub const fn auxdac_2_word_msb(x: u16) -> u8 {
        ((x & 0x3F) << 2) as u8
    }

    /// AuxDAC 1 Word <1:0>
    pub const fn auxdac_1_word(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod auxdac_config {
    pub use arbitrary_int::u2;

    #[bitbybit::bitenum(u2, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum Vref {
        _1V = 0b00,
        _1_5V = 0b01,
        _2V = 0b10,
        _2_5V = 0b11,
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(5, rw)]
        must_be_zero: bool,
        #[bit(4, rw)]
        step_factor: bool,
        #[bits(2..=3, rw)]
        vref: Vref,
        #[bits(0..=1, rw)]
        word_lower_2_bits: u2,
    }
}

pub mod auxadc_config {
    use arbitrary_int::u3;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(1..=3, rw)]
        decimation: u3,
        #[bit(0, rw)]
        power_down: bool,
    }
}

pub mod auxadc_lsb {
    /// AuxADC Word LSB<3:0>
    pub const fn auxadc_word_lsb(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod auto_gpo {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Nibble enable bits for GPO_0 (bit 4) to GPO_3 (bit 7).
        #[bit(4, rw)]
        rx_enable: [bool; 4],
        /// Nibble enable bits for GPO_0 (bit 0) to GPO_3 (bit 3).
        #[bit(0, rw)]
        tx_enable: [bool; 4],
    }
}

pub mod agc_attack_delay {
    pub use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(6, rw)]
        invert_bypassed_lna_polarity: bool,
        #[bits(0..=5, rw)]
        agc_attack_delay_us: u6,
    }
}

pub mod auxdac_enable_ctrl {
    /// AuxDAC Enable Control Register
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(7, rw)]
        disable_aux_dac_2: bool,
        #[bit(6, rw)]
        disable_aux_dac_1: bool,

        #[bit(5, rw)]
        disable_auto_tx_aux_dac_2: bool,
        #[bit(4, rw)]
        disable_auto_tx_aux_dac_1: bool,

        #[bit(3, rw)]
        disable_auto_rx_aux_dac_2: bool,
        #[bit(2, rw)]
        disable_auto_rx_aux_dac_1: bool,

        #[bit(1, rw)]
        disable_dac2_in_alert: bool,
        #[bit(0, rw)]
        disable_dac1_in_alert: bool,
    }
}

pub mod external_lna_ctrl {
    use arbitrary_int::u4;

    /// External LNA Control Register
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// AuxDAC Manual Select
        #[bit(7, rw)]
        auxdac_manual_select: bool,

        /// External LNA2 control.
        ///
        /// The External LNA control bit in the RX2 gain table sets the GPO_1 state.
        #[bit(6, rw)]
        external_lna2_ctrl: bool,

        /// External LNA1 control
        ///
        /// The External LNA control bit in the RX1 gain table sets the GPO_0 state.
        #[bit(5, rw)]
        external_lna1_ctrl: bool,

        /// GPO manual select.
        ///
        /// When clear, the GPOs are slaves to the ENSM.
        #[bit(4, rw)]
        gpo_manual_select: bool,

        /// Open<3:0>
        #[bits(0..=3, rw)]
        open: u4,
    }
}

pub mod gpo_force_and_init {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Nibble enable bits for GPO_0 (bit 4) to GPO_3 (bit 7).
        ///
        /// When clear, GPOs are logic low.
        #[bit(4, rw)]
        manual_control: [bool; 4],
        /// Nibble enable bits for GPO_0 (bit 0) to GPO_3 (bit 3).
        ///
        /// When clear, GPOs are logic low in sleep, wait, and alert states.
        #[bit(0, rw)]
        init_state: [bool; 4],
    }
}

pub mod ctrl_output_enable {
    /// En ctrl7
    pub const EN_CTRL7: u8 = 1 << 7;
    /// En ctrl6
    pub const EN_CTRL6: u8 = 1 << 6;
    /// En ctrl5
    pub const EN_CTRL5: u8 = 1 << 5;
    /// En ctrl4
    pub const EN_CTRL4: u8 = 1 << 4;
    /// En ctrl3
    pub const EN_CTRL3: u8 = 1 << 3;
    /// En ctrl2
    pub const EN_CTRL2: u8 = 1 << 2;
    /// En ctrl1
    pub const EN_CTRL1: u8 = 1 << 1;
    /// En ctrl0
    pub const EN_CTRL0: u8 = 1 << 0;
}

pub mod product_id {
    pub const PRODUCT_ID_MASK: u8 = 0xF8;
    pub const PRODUCT_ID_9361: u8 = 0x08;
    pub const REV_MASK: u8 = 0x07;
}

pub mod reference_clock_cycles {
    /// Reference Clock Cycles per us<6:0>
    pub const fn reference_clock_cycles_per_us(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod digital_io_ctrl {
    /// CLK Out Drive
    pub const CLK_OUT_DRIVE: u8 = 1 << 7;
    /// DATACLK drive
    pub const DATACLK_DRIVE: u8 = 1 << 6;
    /// Data Port Drive
    pub const DATA_PORT_DRIVE: u8 = 1 << 2;

    /// DATACLK slew <1:0>
    pub const fn dataclk_slew(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Data Port Slew<1:0>
    pub const fn data_port_slew(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod lvds_bias_ctrl {
    pub use arbitrary_int::{u2, u3};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// CLK Out Slew<1:0>
        #[bits(6..=7, rw)]
        clk_out_slew: u2,
        /// Rx On Chip Term
        #[bit(5, rw)]
        rx_on_chip_term: bool,
        /// Bypass Bias R
        #[bit(4, rw)]
        lvds_bypass_bias_r: bool,
        /// LVDS Tx LO VCM
        #[bit(3, rw)]
        lvds_tx_lo_vcm: bool,
        /// LVDS Bias<2:0>
        #[bits(0..=2, rw)]
        lvds_bias: u3,
    }
}

pub mod lvds_invert_ctrl1 {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Invert P0\[3:2\]
        #[bit(7, rw)]
        invert_p0_3_2: bool,
        /// Invert P0\[1:0\]
        #[bit(6, rw)]
        invert_p0_1_0: bool,
        /// Invert P1\[11:10\]
        #[bit(5, rw)]
        invert_p1_11_10: bool,
        /// Invert P1\[9:8\]
        #[bit(4, rw)]
        invert_p1_9_8: bool,
        /// Invert P1\[7:6\]
        #[bit(3, rw)]
        invert_p1_7_6: bool,
        /// Invert P1\[5:4\]
        #[bit(2, rw)]
        invert_p1_5_4: bool,
        /// Invert P1\[3:2\]
        #[bit(1, rw)]
        invert_p1_3_2: bool,
        /// Invert P1\[2:0\]
        #[bit(0, rw)]
        invert_p1_2_0: bool,
    }
}

pub mod lvds_invert_ctrl2 {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Invert FBCLK
        #[bit(7, rw)]
        invert_fbclk: bool,
        /// Invert Tx Frame
        #[bit(6, rw)]
        invert_tx_frame: bool,
        /// Invert DATACLK
        #[bit(5, rw)]
        invert_dataclk: bool,
        /// Invert Rx Frame
        #[bit(4, rw)]
        invert_rx_frame: bool,
        /// Invert P0\[11:10\]
        #[bit(3, rw)]
        invert_p0_11_10: bool,
        /// Invert P0\[9:8\]
        #[bit(2, rw)]
        invert_p0_9_8: bool,
        /// Invert P0\[7:6\]
        #[bit(1, rw)]
        invert_p0_7_6: bool,
        /// Invert P0\[5:4\]
        #[bit(0, rw)]
        invert_p0_5_4: bool,
    }
}

pub mod sdm_ctrl_1 {
    /// Init BB FO CAL
    pub const INIT_BB_FO_CAL: u8 = 1 << 2;
    /// BBPLL Reset Bar
    pub const BBPLL_RESET_BAR: u8 = 1 << 0;
}

pub mod clock_ctrl {
    pub type ClockScaler = super::ClockScaler;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(0..=1, rw)]
        scaler: ClockScaler,
    }
}

pub mod cp_bleed_current {
    /// MCS refclk Scale En
    pub const MCS_REFCLK_SCALE_EN: u8 = 1 << 7;
}

pub mod vco_ctrl {
    pub use arbitrary_int::{u2, u3};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(7, rw)]
        freq_cal_enable: bool,
        #[bits(5..=6, rw)]
        should_be_one: u2,
        #[bit(3, rw)]
        force_vco_band_enable: bool,
        #[bits(0..=2, rw)]
        forced_vco_band_word: u3,
    }
}

pub mod sdm_ctrl {
    /// Cal Clock div 4
    pub const CAL_CLOCK_DIV_4: u8 = 1 << 4;
}

pub mod rx_synth_power_down_override {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Rx LO Power Down
        #[bit(4, rw)]
        rx_lo_power_down: bool,
        /// Rx Synth VCO ALC Power Down
        #[bit(3, rw)]
        rx_synth_vco_alc_power_down: bool,
        /// Rx Synth PTAT Power Down
        #[bit(2, rw)]
        rx_synth_ptat_power_down: bool,
        /// Rx Synth VCO Power Down
        #[bit(1, rw)]
        rx_synth_vco_power_down: bool,
        /// Rx Synth VCO LDO Power Down
        #[bit(0, rw)]
        rx_synth_vco_ldo_power_down: bool,
    }
}

pub mod tx_synth_power_down_override {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tx LO Power Down
        #[bit(4, rw)]
        tx_lo_power_down: bool,
        /// Tx Synth VCO ALC Power Down
        #[bit(3, rw)]
        tx_synth_vco_alc_power_down: bool,
        /// Tx Synth PTAT Power Down
        #[bit(2, rw)]
        tx_synth_ptat_power_down: bool,
        /// Tx Synth VCO Power Down
        #[bit(1, rw)]
        tx_synth_vco_power_down: bool,
        /// Tx Synth VCO LDO Power Down
        #[bit(0, rw)]
        tx_synth_vco_ldo_power_down: bool,
    }
}

pub mod rx_analog_power_down_override_1 {
    /// Rx Offset DAC CGin Power Down<1:0>
    pub const fn rx_offset_dac_cgin_power_down(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Rx LMT Overload Power Down<1:0>
    pub const fn rx_lmt_overload_power_down(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Rx Mixer Gm Power Down<1:0>
    pub const fn rx_mixer_gm_power_down(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Rx CGB Power Down<1:0>
    pub const fn rx_cgb_power_down(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod rx_analog_power_down_override_2 {
    /// Rx BBF Power Down<1:0>
    pub const fn rx_bbf_power_down(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Rx TIA Power Down<1:0>
    pub const fn rx_tia_power_down(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Rx Mixer Power Down<1:0>
    pub const fn rx_mixer_power_down(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Rx Offset DAC CGOut Power Down<1:0>
    pub const fn rx_offset_dac_cgout_power_down(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod tx_analog_power_down_override_1 {
    /// Tx Secondary Filter Power Down<1:0>
    pub const fn tx_secondary_filter_power_down(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Tx BBF Power Down<1:0>
    pub const fn tx_bbf_power_down(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Tx DAC Power Down<1:0>
    pub const fn tx_dac_power_down(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Tx DAC Bias Power Down<1:0>
    pub const fn tx_dac_bias_power_down(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod analog_power_down_override {
    /// Rx Ext VCO Buffer Power Down
    pub const RX_EXT_VCO_BUFFER_POWER_DOWN: u8 = 1 << 5;
    /// Tx Ext VCO Buffer Power Down
    pub const TX_EXT_VCO_BUFFER_POWER_DOWN: u8 = 1 << 4;

    /// Tx Monitor Power Down<1:0>
    pub const fn tx_monitor_power_down(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Tx Upconverter Power Down<1:0>
    pub const fn tx_upconverter_power_down(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod misc_power_down_override {
    /// Rx LNA Power Down
    pub const RX_LNA_POWER_DOWN: u8 = 1 << 6;
    /// DCXO Power Down
    pub const DCXO_POWER_DOWN: u8 = 1 << 1;
    /// Master Bias Power Down
    pub const MASTER_BIAS_POWER_DOWN: u8 = 1 << 0;

    /// Rx Calibration Power Down<1:0>
    pub const fn rx_calibration_power_down(x: u8) -> u8 {
        (x & 0x3) << 2
    }
}

pub mod ch_1_overflow {
    /// BBPLL Lock
    pub const BBPLL_LOCK: u8 = 1 << 7;
    /// CH 1 INT3
    pub const CH_1_INT3: u8 = 1 << 6;
    /// CH1 HB3
    pub const CH1_HB3: u8 = 1 << 5;
    /// CH1 HB2
    pub const CH1_HB2: u8 = 1 << 4;
    /// CH1 QEC
    pub const CH1_QEC: u8 = 1 << 3;
    /// CH1 HB1
    pub const CH1_HB1: u8 = 1 << 2;
    /// CH1 TFIR
    pub const CH1_TFIR: u8 = 1 << 1;
    /// CH1 RFIR
    pub const CH1_RFIR: u8 = 1 << 0;
}

pub mod ch_2_overflow {
    /// CH2 INT3
    pub const CH2_INT3: u8 = 1 << 6;
    /// CH2 HB3
    pub const CH2_HB3: u8 = 1 << 5;
    /// CH2 HB2
    pub const CH2_HB2: u8 = 1 << 4;
    /// CH2 QEC
    pub const CH2_QEC: u8 = 1 << 3;
    /// CH2 HB1
    pub const CH2_HB1: u8 = 1 << 2;
    /// CH2 TFIR
    pub const CH2_TFIR: u8 = 1 << 1;
    /// CH2 RFIR
    pub const CH2_RFIR: u8 = 1 << 0;
}

pub mod tx_filter_conf {
    #[bitbybit::bitenum(u3, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum NumberOfTapsRaw {
        _16 = 0,
        _32 = 1,
        _48 = 2,
        _64 = 3,
        _80 = 4,
        _96 = 5,
        _112 = 6,
        _128 = 7,
    }

    impl NumberOfTapsRaw {
        /// Actual number of taps.
        pub const fn number(&self) -> u8 {
            match self {
                NumberOfTapsRaw::_16 => 16,
                NumberOfTapsRaw::_32 => 32,
                NumberOfTapsRaw::_48 => 48,
                NumberOfTapsRaw::_64 => 64,
                NumberOfTapsRaw::_80 => 80,
                NumberOfTapsRaw::_96 => 96,
                NumberOfTapsRaw::_112 => 112,
                NumberOfTapsRaw::_128 => 128,
            }
        }
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(5..=7, rw)]
        number_of_taps: NumberOfTapsRaw,
        #[bit(4, rw)]
        write_goes_to_tx2: bool,
        #[bit(3, rw)]
        write_goes_to_tx1: bool,
        #[bit(2, rw)]
        write_tx: bool,
        #[bit(1, rw)]
        start_tx_clock: bool,
        #[bit(0, rw)]
        attentuate_6db: bool,
    }
}

pub mod tx_mon_low_gain {
    use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tx Mon Track
        #[bit(5, rw)]
        tx_mon_track: bool,
        /// Tx Mon Low Gain<4:0>
        #[bits(0..=4, rw)]
        tx_mon_low_gain: u5,
    }
}

pub mod tx_mon_high_gain {
    use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tx Mon High Gain<4:0>
        #[bits(0..=4, rw)]
        tx_mon_high_gain: u5,
    }
}

pub mod tx_level_thresh {
    use arbitrary_int::u2;
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tx Level Threshold<5:0>
        #[bits(2..=7, rw)]
        tx_level_thresh: u6,
        /// Tx Mon Delay Counter<9:8>
        #[bits(0..=1, rw)]
        tx_mon_delay_counter: u2,
    }
}

pub mod tx_rssi_lsb {
    /// Tx RSSI 2<0>
    pub const TX_RSSI_2: u8 = 1 << 1;
    /// TX RSSI 1<0>
    pub const TX_RSSI_1: u8 = 1 << 0;
}

pub mod tpm_mode_enable {
    use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tx2 Monitor Enable
        #[bit(7, rw)]
        tx2_mon_enable: bool,
        /// One Shot Mode
        #[bit(6, rw)]
        one_shot_mode: bool,
        /// Tx1 Monitor Enable
        #[bit(5, rw)]
        tx1_mon_enable: bool,
        /// Tx Mon Duration<3:0>
        #[bits(0..=3, rw)]
        tx_mon_duration: u4,
    }
}

pub mod tx_mon_config {
    use arbitrary_int::u2;
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tx Mon LO CM<5:0>
        #[bits(2..=7, rw)]
        tx_mon_lo_cm: u6,
        /// Tx Mon Gain<1:0>
        #[bits(0..=1, rw)]
        tx_mon_gain: u2,
    }
}

pub mod tx1_atten_1 {
    /// Tx 1 Atten <8>
    pub const TX_1_ATTEN: u8 = 1 << 0;
}

pub mod tx2_atten_1 {
    /// Tx 2 Atten <8>
    pub const TX_2_ATTEN: u8 = 1 << 0;
}

pub mod tx_atten_offset {
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Mask Clr Atten Update
        #[bit(6, rw)]
        mask_clr_atten_update: bool,
        /// Tx Atten Offset<5:0>
        #[bits(0..=5, rw)]
        tx_atten_offset: u6,
    }
}

pub mod tx1_dig_atten {
    /// Sel Tx1 & Ttx2
    pub const SEL_TX1_TX2: u8 = 1 << 6;
}

pub mod tx2_dig_atten {

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Mask Clr Atten Update
        #[bit(6, rw)]
        immediately_update_tpc_atten: bool,
    }
}

pub mod tx1_symbol_atten {
    /// Tx 1 Symbol Attenuation<6:0>
    pub const fn tx_1_symbol_atten(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod tx2_symbol_atten {
    /// Tx 2 Symbol Attenuation<6:0>
    pub const fn tx_2_symbol_atten(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod tx_symbol_atten_config {
    /// Use Tx1 Pin & Symbol Atten
    pub const USE_TX1_PIN_SYMBOL_ATTEN: u8 = 1 << 3;
    /// Use CTRL IN for symbol Atten
    pub const USE_CTRL_IN_FOR_SYMBOL_ATTEN: u8 = 1 << 1;
    /// Enable Symbol Atten
    pub const ENABLE_SYMBOL_ATTEN: u8 = 1 << 0;
}

pub mod tx_force_bits {
    /// Force Out 2 Tx2 Offset
    pub const FORCE_OUT_2_TX2_OFFSET: u8 = 1 << 7;
    /// Force Out 2 Tx1 Offset
    pub const FORCE_OUT_2_TX1_OFFSET: u8 = 1 << 6;
    /// Force Out 2 Tx2 Phase & Gain
    pub const FORCE_OUT_2_TX2_PHASE_GAIN: u8 = 1 << 5;
    /// Force Out 2 Tx1 Phase & Gain
    pub const FORCE_OUT_2_TX1_PHASE_GAIN: u8 = 1 << 4;
    /// Force Out 1 Tx2 Offset
    pub const FORCE_OUT_1_TX2_OFFSET: u8 = 1 << 3;
    /// Force Out 1 Tx1 Offset
    pub const FORCE_OUT_1_TX1_OFFSET: u8 = 1 << 2;
    /// Force Out 1 Tx2 Phase & Gain
    pub const FORCE_OUT_1_TX2_PHASE_GAIN: u8 = 1 << 1;
    /// Force Out 1 Tx1 Phase & Gain
    pub const FORCE_OUT_1_TX1_PHASE_GAIN: u8 = 1 << 0;
}

pub mod quad_cal_nco_freq_phase_offset {
    use arbitrary_int::u2;
    use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Rx NCO Frequency<1:0>
        #[bits(5..=6, rw)]
        rx_nco_freq: u2,
        /// Rx NCO Phase Offset<4:0>
        #[bits(0..=4, rw)]
        rx_nco_phase_offset: u5,
    }
}

pub mod quad_cal_ctrl {
    use arbitrary_int::u2;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Free Run Enable
        #[bit(7, rw)]
        free_run_enable: bool,
        /// Settle Main Enable
        #[bit(6, rw)]
        settle_main_enable: bool,
        /// DC Offset Enable
        #[bit(5, rw)]
        dc_offset_enable: bool,
        /// Gain Enable
        #[bit(4, rw)]
        gain_enable: bool,
        /// Phase Enable
        #[bit(3, rw)]
        phase_enable: bool,
        /// Quad Cal Soft Reset
        #[bit(2, rw)]
        quad_cal_soft_reset: bool,
        /// M<1:0>
        #[bits(0..=1, rw)]
        m_decim: u2,
    }

    /// Free Run Enable
    pub const FREE_RUN_ENABLE: u8 = 1 << 7;
    /// Settle Main Enable
    pub const SETTLE_MAIN_ENABLE: u8 = 1 << 6;
    /// DC Offset Enable
    pub const DC_OFFSET_ENABLE: u8 = 1 << 5;
    /// Gain Enable
    pub const GAIN_ENABLE: u8 = 1 << 4;
    /// Phase Enable
    pub const PHASE_ENABLE: u8 = 1 << 3;
    /// Quad Cal Soft Reset
    pub const QUAD_CAL_SOFT_RESET: u8 = 1 << 2;

    /// M<1:0>
    pub const fn m_decim(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod kexp_1 {
    use arbitrary_int::u2;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Kexp Tx<1:0>
        #[bits(6..=7, rw)]
        kexp_tx: u2,
        /// Kexp Tx_comp <1:0>
        #[bits(4..=5, rw)]
        kexp_tx_comp: u2,
        /// Kexp DC I <1:0>
        #[bits(2..=3, rw)]
        kexp_dc_i: u2,
        /// Kexp DC Q <1:0>
        #[bits(0..=1, rw)]
        kexp_dc_q: u2,
    }

    /// Kexp Tx<1:0>
    pub const fn kexp_tx(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Kexp Tx_comp <1:0>
    pub const fn kexp_tx_comp(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Kexp DC I <1:0>
    pub const fn kexp_dc_i(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Kexp DC Q <1:0>
    pub const fn kexp_dc_q(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod kexp_2 {
    use arbitrary_int::u2;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tx NCO frequency<1:0>
        #[bits(6..=7, rw)]
        tx_nco_freq: u2,
        /// Invert I data
        #[bit(5, rw)]
        invert_i_data: bool,
        /// Invert Q data
        #[bit(4, rw)]
        invert_q_data: bool,
        /// Kexp Phase <1:0>
        #[bits(2..=3, rw)]
        kexp_phase: u2,
        /// Kexp Amp <1:0>
        #[bits(0..=1, rw)]
        kexp_amp: u2,
    }

    /// Invert I data
    pub const INVERT_I_DATA: u8 = 1 << 5;
    /// Invert Q data
    pub const INVERT_Q_DATA: u8 = 1 << 4;

    /// Tx NCO frequency<1:0>
    pub const fn tx_nco_freq(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Kexp Phase <1:0>
    pub const fn kexp_phase(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Kexp Amp <1:0>
    pub const fn kexp_amp(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod quad_cal_status_tx {
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// TX Convergence Count<5:0>
        #[bits(2..=7, r)]
        tx_convergence_count: u6,
        /// TX LO Conv
        #[bit(1, r)]
        tx_lo_conv: bool,
        /// TX SSB Conv
        #[bit(0, r)]
        tx_ssb_conv: bool,
    }
}

pub mod tx_quad_full_lmt_gain {
    use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// RX Full table/LMT table gain<6:0>
        #[bits(0..=6, rw)]
        rx_full_tablelmt_table_gain: u7,
    }

    /// RX Full table/LMT table gain<6:0>
    pub const fn rx_full_tablelmt_table_gain(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod squarer_config {
    /// Gm Stage Time Con Override
    pub const GM_STAGE_TIME_CON_OVERRIDE: u8 = 1 << 5;
    /// Gm Stage MV HP Pole
    pub const GM_STAGE_MV_HP_POLE: u8 = 1 << 4;
    /// Gm Stage Lower CM
    pub const GM_STAGE_LOWER_CM: u8 = 1 << 3;
    /// Bypass Bias R
    pub const BYPASS_BIAS_R: u8 = 1 << 0;

    /// Vbias Control<1:0>
    pub const fn vbias_ctrl(x: u8) -> u8 {
        (x & 0x3) << 1
    }
}

pub mod thresh_accum {
    /// Threshold Accumulator<3:0>
    pub const fn thresh_accumulator(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod tx_quad_lpf_gain {
    use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// RX LPF gain<4:0>
        #[bits(0..=4, rw)]
        rx_lpf_gain: u5,
    }

    /// RX LPF gain<4:0>
    pub const fn rx_lpf_gain(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod txdac_vds_i {
    /// TxDAC Vds I<5:0>
    pub const fn txdac_vds_i(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod txdac_vds_q {
    /// TxDAC Vds Q<5:0>
    pub const fn txdac_vds_q(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod txdac_gn_i {
    /// txDAC_gn_I<5:0>
    pub const fn txdac_gn_i(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod txdac_gn_q {
    /// txDAC_gn_Q<5:0>
    pub const fn txdac_gn_q(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod txbbf_opamp_a {
    /// OpAmpA Output Bias<1:0>
    pub const fn opampa_output_bias(x: u8) -> u8 {
        (x & 0x3) << 5
    }

    /// OpAmpA RZ<1:0>
    pub const fn opampa_rz(x: u8) -> u8 {
        (x & 0x3) << 3
    }

    /// OpAmp A CC<2:0>
    pub const fn opamp_a_cc(x: u8) -> u8 {
        x & 0x7
    }
}

pub mod txbbf_opamp_b {
    /// OpAmpB Output Bias<1:0>
    pub const fn opampb_output_bias(x: u8) -> u8 {
        (x & 0x3) << 5
    }

    /// OpAmpB RZ<1:0>
    pub const fn opampb_rz(x: u8) -> u8 {
        (x & 0x3) << 3
    }

    /// OpAmp B CC<2:0>
    pub const fn opamp_b_cc(x: u8) -> u8 {
        x & 0x7
    }
}

pub mod tx_bbf_r1 {
    /// Override enable
    pub const OVERRIDE_ENABLE: u8 = 1 << 7;

    /// R1<4:0>
    pub const fn r1(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod tx_bbf_r2 {
    /// R2<4:0>
    pub const fn r2(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod tx_bbf_r3 {
    /// R3<4:0>
    pub const fn r3(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod tx_bbf_r4 {
    /// R4<4:0>
    pub const fn r4(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod tx_bbf_rp {
    /// Rp<4:0>
    pub const fn rp(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod tx_bbf_c1 {
    /// C1<5:0>
    pub const fn c1(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod tx_bbf_c2 {
    /// C2<5:0>
    pub const fn c2(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod tx_bbf_cp {
    /// Cp<5:0>
    pub const fn cp(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod tx_tune_ctrl {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tune Control<1:0>
        #[bits(5..=6, rw)]
        tune_ctrl: arbitrary_int::u2,
        /// PD Tune
        #[bit(2, rw)]
        pd_tune: bool,
        /// Tuner Resample
        #[bit(1, rw)]
        tuner_resample: bool,
        /// Tuner Resample Phase
        #[bit(0, rw)]
        tuner_resample_phase: bool,
    }
}

pub mod tx_bbf_r2b {
    /// Bypass Bias R
    pub const TX_BBF_BYPASS_BIAS_R: u8 = 1 << 7;
    /// R2b Ovr
    pub const R2B_OVR: u8 = 1 << 5;

    /// R2b<4:0>
    pub const fn r2b(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod tx_bbf_tune {
    /// BBF1 Comp I
    pub const BBF1_COMP_I: u8 = 1 << 3;
    /// BBF1 Comp Q
    pub const BBF1_COMP_Q: u8 = 1 << 2;
    /// BBF2 Comp I
    pub const BBF2_COMP_I: u8 = 1 << 1;
    /// BBF2 Comp Q
    pub const BBF2_COMP_Q: u8 = 1 << 0;
}

pub mod config0 {
    use arbitrary_int::u2;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Bias<1:0>
        #[bits(6..=7, rw)]
        bias: u2,
        /// Rgm<1:0>
        #[bits(4..=5, rw)]
        rgm: u2,
        /// Cc<1:0>
        #[bits(2..=3, rw)]
        cc: u2,
        /// AmpBias<1:0>
        #[bits(0..=1, rw)]
        ampbias: u2,
    }
}

pub mod resistor {
    use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Resistor<3:0>
        #[bits(0..=3, rw)]
        resistor: u4,
    }
}

pub mod capacitor {
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Capacitor<5:0>
        #[bits(0..=5, rw)]
        capacitor: u6,
    }
}

pub mod lo_cm {
    /// LO Common Mode<1:0>
    pub const fn lo_common_mode(x: u8) -> u8 {
        (x & 0x3) << 5
    }
}

pub mod tx_bbf_tune_mode {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tune Comp Mask<1:0>
        #[bits(5..=6, rw)]
        tune_comp_mask: arbitrary_int::u2,
        /// EvalTime
        #[bit(4, rw)]
        evaltime: bool,
        /// Tuner Mode<2:0>
        #[bits(1..=3, rw)]
        tuner_mode: arbitrary_int::u3,
        /// TX BBF Tune Divider<8>
        #[bit(0, rw)]
        tx_bbf_tune_divider_msb: bool,
    }
}

pub mod rx_filter_config {
    use super::tx_filter_conf::NumberOfTapsRaw;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(5..=7, rw)]
        number_of_taps: NumberOfTapsRaw,
        #[bit(4, rw)]
        write_goes_to_rx2: bool,
        #[bit(3, rw)]
        write_goes_to_rx1: bool,
        #[bit(2, rw)]
        write_rx: bool,
        #[bit(1, rw)]
        start_rx_clock: bool,
    }
}

pub mod rx_filter_gain {
    /// Filter gain, `REG_RX_FILTER_GAIN`. The C driver derives the raw value from a dB figure
    /// via `3 - (gain_dB + 12) / 6`, which inverts the naive bit order: `Plus6dB` = 0, `Zero` =
    /// 1, `Minus6dB` = 2, `Minus12dB` = 3.
    #[bitbybit::bitenum(u2, exhaustive = true)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Debug, PartialEq, Eq)]
    pub enum FilterGain {
        Plus6dB = 0b00,
        Zero = 0b01,
        Minus6dB = 0b10,
        Minus12dB = 0b11,
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(0..=1, rw)]
        filter_gain: FilterGain,
    }
}

pub mod agc_config_1 {
    #[bitbybit::bitenum(u2, exhaustive = true)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Debug, PartialEq, Eq)]
    pub enum RxGainControlSetup {
        /// MGC
        Manual = 0x00,
        /// AGC Fast Attack
        AutomaticFastAttack = 0x01,
        /// AGC Slow Attack
        AutomaticSlowAttack = 0x02,
        /// AGC Slow Attack Hybrid
        AutomaticSlowAttackHybrid = 0x03,
    }

    impl From<crate::types::GainControlMode> for RxGainControlSetup {
        fn from(value: crate::types::GainControlMode) -> Self {
            match value {
                crate::types::GainControlMode::Manual => RxGainControlSetup::Manual,
                crate::types::GainControlMode::AutoFastAttack => {
                    RxGainControlSetup::AutomaticFastAttack
                }
                crate::types::GainControlMode::AutoSlowAttack => {
                    RxGainControlSetup::AutomaticSlowAttack
                }
                crate::types::GainControlMode::AutoHybrid => {
                    RxGainControlSetup::AutomaticSlowAttackHybrid
                }
            }
        }
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Dec Pwr for Low Pwr
        #[bit(7, rw)]
        dec_pwr_for_low_pwr: bool,
        /// Dec Pwr for Lock Level
        #[bit(6, rw)]
        dec_pwr_for_lock_level: bool,
        /// Dec Pwr for Gain Lock Exit
        #[bit(5, rw)]
        dec_pwr_for_gain_lock_exit: bool,
        /// Slow Attack Hybrid Mode
        #[bit(4, rw)]
        slow_attack_hybrid_mode: bool,
        /// Rx 2 Gain Control Setup<1:0>
        #[bits(2..=3, rw)]
        rx2_gain_ctrl_setup: RxGainControlSetup,
        /// Rx 1 Gain Control Setup<1:0>
        #[bits(0..=1, rw)]
        rx1_gain_ctrl_setup: RxGainControlSetup,
    }
}

pub mod agc_config_2 {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Soft Reset
        #[bit(7, rw)]
        agc_soft_reset: bool,
        /// Gain Unlock Control
        #[bit(6, rw)]
        agc_gain_unlock_ctrl: bool,
        /// Use Full Gain Table
        #[bit(3, rw)]
        agc_use_full_gain_table: bool,
        /// Enable Digital Gain
        #[bit(2, rw)]
        dig_gain_en: bool,
        /// Manual Gain Control Rx 2
        #[bit(1, rw)]
        man_gain_ctrl_rx2: bool,
        /// Manual Gain Control Rx 1
        #[bit(0, rw)]
        man_gain_ctrl_rx1: bool,
    }
}

pub mod agc_config_3 {
    pub use arbitrary_int::u3;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Manual (CTRL_IN) Incr Gain Step Size<2:0>
        #[bits(5..=7, rw)]
        manual_incr_step_size: u3,
        /// Inc/Dec LMT Gain
        #[bit(4, rw)]
        incdec_lmt_gain: bool,
        /// Use AGC for LMT/LPF Gain
        #[bit(3, rw)]
        use_agc_for_lmtlpf_gain: bool,
        /// ADC Overrange Sample Size<2:0>
        #[bits(0..=2, rw)]
        adc_overrange_sample_size: u3,
    }
}

pub mod max_lmt_full_gain {
    pub use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Maximum Full Table/LMT Table Index<6:0>
        #[bits(0..=6, rw)]
        maximum_full_tablelmt_table_index: u7,
    }
}

pub mod peak_wait_time {
    pub use arbitrary_int::{u3, u5};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(5..=7, rw)]
        manual_decr_gain_stp_size: u3,
        #[bits(0..=4, rw)]
        peak_overload_wait_time: u5,
    }
}

pub mod digital_gain {
    use arbitrary_int::{u3, u5};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(5..=7, rw)]
        dig_gain_step_size: u3,
        #[bits(0..=4, rw)]
        maximum_digital_gain: u5,
    }
}

pub mod agc_lock_level {
    pub use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Enable Dig Sat Ovrg
        #[bit(7, rw)]
        enable_dig_sat_ovrg: bool,
        /// AGC Lock Level (Fast)/ AGC Inner High Threshold (Slow) <6:0>
        #[bits(0..=6, rw)]
        agc_lock_level_fast_agc_inner_high_thresh_slow: u7,
    }
}

pub mod gain_step_config1 {
    pub use arbitrary_int::{u2, u3};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// LMT Detector Settling Time<2:0>
        #[bits(5..=7, rw)]
        lmt_detector_settling_time: u3,
        /// Dec Step Size for: Large LMT Overload/ Full Table Case #3 <2:0>
        #[bits(2..=4, rw)]
        dec_stp_size_for_large_lmt_overload: u3,
        /// ADC Noise Correction Factor<1:0>
        #[bits(0..=1, rw)]
        adc_noise_correction_factor: u2,
    }
}

pub mod gain_step_config2 {
    pub use arbitrary_int::{u3, u4};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Fast Attack Only. Decrement Step Size for: Small LPF Gain Change / Full Table Case #2 <2:0>
        #[bits(4..=6, rw)]
        decrement_stp_size_for_small_lpf_gain_change: u3,
        /// Decrement Step Size for: Large LPF Gain Change / Full Table Case #1<3:0>
        #[bits(0..=3, rw)]
        large_lpf_gain_step: u4,
    }
}

pub mod small_lmt_overload_thresh {
    pub use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Force PD Reset Rx2
        #[bit(7, rw)]
        force_pd_reset_rx2: bool,
        /// Force PD Reset Rx1
        #[bit(6, rw)]
        force_pd_reset_rx1: bool,
        /// Small LMT Overload Threshold<5:0>
        #[bits(0..=5, rw)]
        small_lmt_overload_thresh: u6,
    }
}

pub mod large_lmt_overload_thresh {
    /// Large LMT Overload Threshold<5:0>
    pub const fn large_lmt_overload_thresh(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod rx1_manual_lmt_full_gain {
    pub use arbitrary_int::{u1, u7};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Power Meas in State 5 <3>
        #[bit(7, rw)]
        power_meas_in_state_5_msb: bool,
        /// Rx1 Manual Full table/LMT table Gain Index<6:0>
        #[bits(0..=6, rw)]
        rx1_manual_full_table_lmt_table_gain_index: u7,
    }
}

pub mod rx1_manual_lpf_gain {
    pub use arbitrary_int::{u3, u5};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Power Meas in State 5<2:0>
        #[bits(5..=7, rw)]
        power_meas_in_state_5: u3,
        /// Rx1 Manual LPF Gain <4:0>
        #[bits(0..=4, rw)]
        rx1_manual_lpf_gain: u5,
    }
}

pub mod rx1_manual_digitalforced_gain {
    /// Force Rx1 Digital Gain
    pub const FORCE_RX1_DIGITAL_GAIN: u8 = 1 << 5;

    /// Rx1 Manual/Forced Digital Gain<4:0>
    pub const fn rx1_manualforced_digital_gain(x: u8) -> u8 {
        x & 0x1F
    }

    pub const RX_DIGITAL_IDX_MASK: u8 = 0x1F;
}

pub mod rx2_manual_lmt_full_gain {
    /// Rx2 Manual Full table/ LMT table Gain Index<6:0>
    pub const fn rx2_manual_full_table_lmt_table_gain_index(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx2_manual_lpf_gain {
    /// Rx2 Manual LPF Gain<4:0>
    pub const fn rx2_manual_lpf_gain(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod rx2_manual_digitalforced_gain {
    /// Force Rx2 Digital Gain
    pub const FORCE_RX2_DIGITAL_GAIN: u8 = 1 << 5;

    /// Rx2 Manual/Forced Digital Gain<4:0>
    pub const fn rx2_manualforced_digital_gain(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod fast_config_1 {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Enable Gain Inc after Gain Lock
        #[bit(7, rw)]
        enable_gain_inc_after_gain_lock: bool,
        /// Goto Opt Gain if Energy Lost or EN_AGC High
        #[bit(6, rw)]
        goto_opt_gain_if_energy_lost_or_en_agc_high: bool,
        /// Goto Set Gain if EN_AGC High
        #[bit(5, rw)]
        goto_set_gain_if_en_agc_high: bool,
        /// Goto Set Gain if Exit Rx State
        #[bit(4, rw)]
        goto_set_gain_if_exit_rx_state: bool,
        /// Don't Unlock Gain if Energy Lost
        #[bit(3, rw)]
        dont_unlock_gain_if_energy_lost: bool,
        /// Goto Optimized Gain if Exit Rx State
        #[bit(2, rw)]
        goto_optimized_gain_if_exit_rx_state: bool,
        /// Don't Unlock Gain If Lg ADC or LMT Ovrg
        #[bit(1, rw)]
        dont_unlock_gain_if_lg_adc_or_lmt_ovrg: bool,
        /// Enable Incr Gain
        #[bit(0, rw)]
        enable_incr_gain: bool,
    }
}

pub mod fast_config_2_settling_delay {
    pub use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Use Last Lock Level for Set Gain
        #[bit(7, rw)]
        use_last_lock_level_for_set_gain: bool,
        /// Enable LMT Gain Inc for Lock Level
        #[bit(6, rw)]
        enable_lmt_gain_inc_for_lock_level: bool,
        /// Goto Max Gain or Opt Gain if EN_AGC High
        #[bit(5, rw)]
        goto_max_gain_or_opt_gain_if_en_agc_high: bool,
        /// Settling Delay<4:0>
        #[bits(0..=4, rw)]
        settling_delay: u5,
    }
}

pub mod fast_energy_lost_thresh {
    pub use arbitrary_int::{u2, u6};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Post Lock Level Step Size for: LPF Table/ Full Table <1:0>
        #[bits(6..=7, rw)]
        post_lock_level_stp_size_for_lpf_table_full_table: u2,
        /// Energy lost threshold<5:0>
        #[bits(0..=5, rw)]
        energy_lost_thresh: u6,
    }
}

pub mod fast_stronger_signal_thresh {
    pub use arbitrary_int::{u2, u6};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Post Lock Level Step for LMT Table <1:0>
        #[bits(6..=7, rw)]
        post_lock_level_stp_for_lmt_table: u2,
        /// Stronger Signal Threshold<5:0>
        #[bits(0..=5, rw)]
        stronger_signal_thresh: u6,
    }
}

pub mod fast_low_power_thresh {
    pub use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Don't unlock gain if ADC Ovrg
        #[bit(7, rw)]
        dont_unlock_gain_if_adc_ovrg: bool,
        /// Low Power Threshold<6:0>
        #[bits(0..=6, rw)]
        low_power_thresh: u7,
    }
}

pub mod fast_strong_signal_freeze {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Don't unlock gain if Stronger Signal
        #[bit(7, rw)]
        dont_unlock_gain_if_stronger_signal: bool,
    }
}

pub mod fast_final_over_range_and_opt_gain {
    pub use arbitrary_int::{u3, u4};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Final Over Range Count<2:0>
        #[bits(5..=7, rw)]
        final_over_range_count: u3,
        /// Optimize Gain Offset<3:0>
        #[bits(0..=3, rw)]
        optimize_gain_offset: u4,
    }
}

pub mod fast_energy_detect_count {
    pub use arbitrary_int::{u3, u5};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Increment Gain Step (LPF/LMT)<2:0>
        #[bits(5..=7, rw)]
        increment_gain_stp_lpflmt: u3,
        /// Energy Detect count<4:0>
        #[bits(0..=4, rw)]
        energy_detect_count: u5,
    }
}

pub mod fast_agcll_upper_limit {
    pub use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// AGCLL Max Increase<5:0>
        #[bits(0..=5, rw)]
        agcll_max_increase: u6,
    }
}

pub mod fast_gain_lock_exit_count {
    pub use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Gain Lock Exit Count<5:0>
        #[bits(0..=5, rw)]
        gain_lock_exit_count: u6,
    }
}

pub mod fast_initial_lmt_gain_limit {
    /// Initial LMT Gain Limit<6:0>
    pub const fn initial_lmt_gain_limit(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod agc_inner_low_thresh {
    pub use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Prevent Gain Inc
        #[bit(7, rw)]
        prevent_gain_inc: bool,
        /// AGC Inner Low Threshold<6:0>
        #[bits(0..=6, rw)]
        agc_inner_low_thresh: u7,
    }
}

pub mod lmt_overload_counters {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Large LMT Overload Exceeded Counter<3:0>
        #[bits(4..=7, rw)]
        large_lmt_overload_exed_counter: u4,
        /// Small LMT Overload Exceeded Counter<3:0>
        #[bits(0..=3, rw)]
        small_lmt_overload_exed_counter: u4,
    }
}

pub mod adc_overload_counters {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Large ADC Overload Exceeded Counter<3:0>
        #[bits(4..=7, rw)]
        large_adc_overload_exed_counter: u4,
        /// Small ADC Overload Exceeded Counter<3:0>
        #[bits(0..=3, rw)]
        small_adc_overload_exed_counter: u4,
    }
}

pub mod gain_step1 {
    pub use arbitrary_int::u3;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Immed. Gain Change if Lg LMT Overload
        #[bit(7, rw)]
        immed_gain_change_if_lg_lmt_overload: bool,
        /// AGC Inner High Threshold Exceeded Step Size<2:0>
        #[bits(4..=6, rw)]
        agc_inner_high_thresh_exed_stp_size: u3,
        /// Immed. Gain Change if Lg ADC Overload
        #[bit(3, rw)]
        immed_gain_change_if_lg_adc_overload: bool,
        /// AGC Inner Low Threshold Exceeded Step Size<2:0>
        #[bits(0..=2, rw)]
        agc_inner_low_thresh_exed_stp_size: u3,
    }
}

pub mod digital_sat_counter {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Double Gain Counter
        #[bit(5, rw)]
        double_gain_counter: bool,
        /// Enable Sync for Gain Counter
        #[bit(4, rw)]
        enable_sync_for_gain_counter: bool,
        /// Dig Saturation Exceeded Counter<3:0>
        #[bits(0..=3, rw)]
        dig_saturation_exed_counter: u4,
    }
}

pub mod outer_power_threshs {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// AGC Outer High Threshold<3:0>
        #[bits(4..=7, rw)]
        agc_offset_of_outer_high_to_inner_high: u4,
        /// AGC Outer Low Threshold<3:0>
        #[bits(0..=3, rw)]
        agc_offset_to_outer_low_to_inner_low: u4,
    }
}

pub mod gain_step2 {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// AGC outer High Threshold Exceeded Step Size<3:0>
        #[bits(4..=7, rw)]
        agc_outer_high_thresh_exed_stp_size: u4,
        /// AGC Outer Low Threshold Exceeded Step Size<3:0>
        #[bits(0..=3, rw)]
        agc_outer_low_thresh_exed_stp_size: u4,
    }
}

pub mod ext_lna_high_gain {
    /// Ext LNA High Gain<5:0>
    pub const fn ext_lna_high_gain(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod ext_lna_low_gain {
    /// Ext LNA Low Gain<5:0>
    pub const fn ext_lna_low_gain(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod gain_table_address {
    pub use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Gain Table Address<6:0>
        #[bits(0..=6, rw)]
        gain_table_address: u7,
    }
}

pub mod gain_table_write_data1 {
    pub use arbitrary_int::{u2, u5};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Ext LNA Ctrl
        #[bit(7, rw)]
        ext_lna_ctrl: bool,
        /// LNA Gain <1:0>
        #[bits(5..=6, rw)]
        lna_gain: u2,
        /// Mixer Gm Gain <4:0>
        #[bits(0..=4, rw)]
        mixer_gm_gain: u5,
    }
}

pub mod gain_table_write_data2 {
    pub use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// TIA Gain
        #[bit(5, rw)]
        tia_gain: bool,
        /// LPF Gain <4:0>
        #[bits(0..=4, rw)]
        lpf_gain: u5,
    }
}

pub mod gain_table_write_data3 {
    pub use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// RF DC Cal
        #[bit(5, rw)]
        rf_dc_cal: bool,
        /// Digital Gain <4:0>
        #[bits(0..=4, rw)]
        digital_gain: u5,
    }
}

pub mod gain_table_read_data_1 {
    pub use arbitrary_int::{u2, u5};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// LNA Gain <1:0>
        #[bits(5..=6, rw)]
        lna_gain: u2,
        /// Mixer Gm Gain <4:0>
        #[bits(0..=4, rw)]
        mixer_gm_gain: u5,
    }
}

pub mod gain_table_read_data_2 {
    pub use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// LPF Gain <4:0>
        #[bits(0..=4, rw)]
        lpf_gain: u5,
    }
}

pub mod gain_table_read_data_3 {
    pub use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Digital Gain <4:0>
        #[bits(0..=4, rw)]
        digital_gain: u5,
    }
}

pub mod gain_table_config {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(4, rw)]
        select_rx2: bool,
        #[bit(3, rw)]
        select_rx1: bool,
        /// Write Gain Table
        #[bit(2, rw)]
        write_gain_table: bool,
        /// Start Gain Table Clock
        #[bit(1, rw)]
        start_gain_table_clock: bool,
    }
}

pub mod gm_sub_table_gain_write {
    pub use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Gm Sub Table Gain Word Write<6:0>
        #[bits(0..=6, rw)]
        gm_sub_table_gain_write: u7,
    }
}

pub mod gm_sub_table_bias_write {
    pub use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Gm Sub Table Bias Word Write<4:0>
        #[bits(0..=4, rw)]
        gm_sub_table_bias_write: u5,
    }
}

pub mod gm_sub_table_ctrl_write {
    pub use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Gm Sub Table Control Word Write<5:0>
        #[bits(0..=5, rw)]
        gm_sub_table_ctrl_write: u6,
    }
}

pub mod gm_sub_table_gain_read {
    pub use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Gm Sub Table Gain Word Read<6:0>
        #[bits(0..=6, rw)]
        gm_sub_table_gain_read: u7,
    }
}

pub mod gm_sub_table_bias_read {
    pub use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Gm Sub Table Bias Word Read<4:0>
        #[bits(0..=4, rw)]
        gm_sub_table_bias_read: u5,
    }
}

pub mod gm_sub_table_ctrl_read {
    pub use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Gm Sub Table Control Word Read<5:0>
        #[bits(0..=5, rw)]
        gm_sub_table_ctrl_read: u6,
    }
}

pub mod gm_sub_table_config {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Write Gm Sub Table
        #[bit(2, rw)]
        write_gm_sub_table: bool,
        /// Start Gm Sub Table Clock
        #[bit(1, rw)]
        start_gm_sub_table_clock: bool,
    }
}

pub mod gain_diff_worderror_write {
    /// Calib Table Gain Diff/Error Word<5:0>
    pub const fn calib_table_gain_differror_word(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod gain_error_read {
    /// Calib Table Gain Error<4:0>
    pub const fn calib_table_gain_error(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod config {
    /// Read Select
    pub const READ_SELECT: u8 = 1 << 4;
    /// Write Mixer Error Table
    pub const WRITE_MIXER_ERROR_TABLE: u8 = 1 << 3;
    /// Write LNA Error Table
    pub const WRITE_LNA_ERROR_TABLE: u8 = 1 << 2;
    /// Write LNA Gain Diff
    pub const WRITE_LNA_GAIN_DIFF: u8 = 1 << 1;
    /// Start Calib Table Clock
    pub const START_CALIB_TABLE_CLOCK: u8 = 1 << 0;

    /// Calib Table Select<1:0>
    pub const fn calib_table_select(x: u8) -> u8 {
        (x & 0x3) << 5
    }
}

pub mod lna_gain_diff_read_back {
    /// LNA Calib Table Gain Difference Word<5:0>
    pub const fn lna_calib_table_gain_difference_word(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod max_mixer_calibration_gain_index {
    /// Max Mixer Calibration Gain Index<4:0>
    pub const fn max_mixer_calibration_gain_index(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod settle_time {
    /// Enable Dig Gain Corr
    pub const ENABLE_DIG_GAIN_CORR: u8 = 1 << 7;
    /// Force Temp Sensor for Cal
    pub const FORCE_TEMP_SENSOR_FOR_CAL: u8 = 1 << 6;

    /// Settle Time<5:0>
    pub const fn settle_time(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod measure_duration {
    /// Gain Cal Meas Duration<3:0>
    pub const fn gain_cal_meas_duration(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod measure_duration_01 {
    use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// RSSI Measurement Duration 1
        #[bits(4..=7, rw)]
        duration_1: u4,
        /// RSSI Measurement Duration 0
        #[bits(0..=3, rw)]
        duration_0: u4,
    }
}

pub mod measure_duration_23 {
    use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// RSSI Measurement Duration 3
        #[bits(4..=7, rw)]
        duration_3: u4,
        /// RSSI Measurement Duration 2
        #[bits(0..=3, rw)]
        duration_2: u4,
    }
}

pub mod rssi_config {
    use arbitrary_int::u2;

    /// RSSI restart/trigger mode
    #[bitbybit::bitenum(u3, exhaustive = false)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum RestartMode {
        AgcInFastAttackModeLocksTheGain = 0,
        EnAgcPinIsPulledHigh = 1,
        EntersRxMode = 2,
        GainChangeOccurs = 3,
        SpiWriteToRegister = 4,
        GainChangeOccursOrEnAgcPinPulledHigh = 5,
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// RFIR for RSSI measurement
        #[bits(6..=7, rw)]
        rfir_for_rssi_measurement: u2,
        /// Start RSSI Meas (only used in SpiWriteToRegister mode)
        #[bit(5, rw)]
        start_rssi_meas: bool,
        /// RSSI Mode Select
        #[bits(2..=4, rw)]
        restart_mode: Option<RestartMode>,
        /// Must be 0
        #[bit(1, rw)]
        should_be_zero: bool,
        /// Default RSSI Meas Mode (power-of-two duration)
        #[bit(0, rw)]
        default_rssi_meas_mode: bool,
    }
}

pub mod adc_measure_duration_01 {
    /// ADC Power Measurement Duration 1<3:0>
    pub const fn adc_power_measurement_duration_1(x: u8) -> u8 {
        (x & 0xF) << 4
    }

    /// ADC Power Measurement Duration 0 <3:0>
    pub const fn adc_power_measurement_duration_0(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod dec_power_measure_duration {
    pub use arbitrary_int::u4;

    /// Decimation power measurement source selection.
    #[bitbybit::bitenum(u1, exhaustive = true)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum DecPowerMeasurementSource {
        RxFirOut = 0,
        Hb1Out = 1,
    }

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Use HB1 Out or Rx FIR Out for Dec pwr Meas
        #[bit(6, rw)]
        dec_power_measurement_source: DecPowerMeasurementSource,
        /// Enable Dec Pwr Meas
        #[bit(5, rw)]
        enable_dec_pwr_meas: bool,
        /// Default Mode ADC Power
        #[bit(4, rw)]
        default_mode_adc_power: bool,
        /// Dec Power Measurement Duration <3:0>
        #[bits(0..=3, rw)]
        dec_power_measurement_duration: u4,
    }
}

pub mod lna_gain {
    /// dB Gain Read-back Channel
    pub const DB_GAIN_READBACK_CHANNEL: u8 = 1 << 0;

    /// Max LNA Gain<6:0>
    pub const fn max_lna_gain(x: u8) -> u8 {
        (x & 0x7F) << 1
    }
}

pub mod rx_quad_cal_level {
    /// Rx Quad Cal Level <3 :0>
    pub const fn rx_quad_cal_level(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod calibration_config_1 {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Enable Phase Corr
        #[bit(7, rw)]
        enable_phase_corr: bool,
        /// Enable Gain Corr
        #[bit(6, rw)]
        enable_gain_corr: bool,
        /// Use Settle Count for DC Cal Wait
        #[bit(5, rw)]
        use_settle_count_for_dc_cal_wait: bool,
        /// Fixed DC Cal Wait Time
        #[bit(4, rw)]
        fixed_dc_cal_wait_time: bool,
        /// Free Run Mode
        #[bit(3, rw)]
        free_run_mode: bool,
        /// Enable Corr Word Decimation
        #[bit(2, rw)]
        enable_corr_word_decimation: bool,
        /// Enable Tracking Mode CH2
        #[bit(1, rw)]
        enable_tracking_mode_ch2: bool,
        /// Enable Tracking Mode CH1
        #[bit(0, rw)]
        enable_tracking_mode_ch1: bool,
    }
}

pub mod calibration_config_2 {
    use arbitrary_int::u2;
    use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Soft Reset
        #[bit(7, rw)]
        soft_reset: bool,
        /// Must be 2'b11
        #[bits(5..=6, rw)]
        should_be_ones: u2,
        /// K exp Phase<4:0>
        #[bits(0..=4, rw)]
        k_exp_phase: u5,
    }
}

pub mod calibration_config_3 {
    use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Prevent Pos Loop Gain
        #[bit(7, rw)]
        prevent_pos_loop_gain: bool,
        /// K exp Amplitude<4:0>
        #[bits(0..=4, rw)]
        k_exp_amplitude: u5,
    }
}

pub mod rx_quad_gain1 {
    /// Rx Full table/LMT table gain<6:0>
    pub const fn rx_full_tablelmt_table_gain(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx_quad_gain2 {
    use arbitrary_int::u3;
    use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Correction Word Decimation M<2:0>
        #[bits(5..=7, rw)]
        correction_word_decimation_m: u3,
        /// Rx LPF gain<4:0>
        #[bits(0..=4, rw)]
        rx_lpf_gain: u5,
    }
}

pub mod rx1_input_a_offsets {
    /// Rx1 Input A "I" DC Offset<5:0>
    pub const fn rx1_input_a_i_dc_offset_lsb(x: u8) -> u8 {
        (x & 0x3F) << 2
    }

    /// Rx1 Input A "Q" DC Offset<9:8>
    pub const fn rx1_input_a_q_dc_offset(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod input_a_offsets_1 {
    /// Rx2 Input A "Q" DC Offset<3:0>
    pub const fn rx2_input_a_q_dc_offset_lsb(x: u8) -> u8 {
        (x & 0xF) << 4
    }

    /// Rx1 Input A "I" DC Offset<9:6>
    pub const fn rx1_input_a_i_dc_offset_msb(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod rx2_input_a_offsets {
    /// Rx2 Input A "I" DC Offset<1:0>
    pub const fn rx2_input_a_i_dc_offset(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Rx2 Input A "Q" DC Offset<9:4>
    pub const fn rx2_input_a_q_dc_offset_msb(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod rx1_input_bc_offsets {
    /// Rx1 Input B&C "I" DC Offset<5:0>
    pub const fn rx1_input_bc_i_dc_offset_lsb(x: u8) -> u8 {
        (x & 0x3F) << 2
    }

    /// Rx1 Input B&C "Q" DC Offset<9:8>
    pub const fn rx1_input_bc_q_dc_offset(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod input_bc_offsets_1 {
    /// Rx2 Input B&C "Q" DC Offset<3:0>
    pub const fn rx2_input_bc_q_dc_offset_lsb(x: u8) -> u8 {
        (x & 0xF) << 4
    }

    /// Rx1 Input B&C "I" DC Offset<9:6>
    pub const fn rx1_input_bc_i_dc_offset_msb(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod rx2_input_bc_offsets {
    /// Rx2 Input B&C "I" DC Offset<1:0>
    pub const fn rx2_input_bc_i_dc_offset(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Rx2 Input B&C "Q" DC Offset<9:4>
    pub const fn rx2_input_bc_q_dc_offset_msb(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod force_bits {
    /// Rx2 Input B&C Force offset
    pub const RX2_INPUT_BC_FORCE_OFFSET: u8 = 1 << 7;
    /// Rx1 Input B&C Force offset
    pub const RX1_INPUT_BC_FORCE_OFFSET: u8 = 1 << 6;
    /// Rx2 Input B&C Force Ph/Gain
    pub const RX2_INPUT_BC_FORCE_PHGAIN: u8 = 1 << 5;
    /// Rx1 Input B&C Force Ph/Gain
    pub const RX1_INPUT_BC_FORCE_PHGAIN: u8 = 1 << 4;
    /// Rx2 Input A Force offset
    pub const RX2_INPUT_A_FORCE_OFFSET: u8 = 1 << 3;
    /// Rx1 Input A Force offset
    pub const RX1_INPUT_A_FORCE_OFFSET: u8 = 1 << 2;
    /// Rx2 Input A Force Ph/Gain
    pub const RX2_INPUT_A_FORCE_PHGAIN: u8 = 1 << 1;
    /// Rx1 Input A Force Ph/Gain
    pub const RX1_INPUT_A_FORCE_PHGAIN: u8 = 1 << 0;
}

pub mod wait_count {
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Wait Count<5:0>
        #[bits(0..=5, rw)]
        wait_count: u6,
    }
}

pub mod rf_dc_offset_config_1 {
    use arbitrary_int::u2;
    use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// DAC FS<1:0>
        #[bits(4..=5, rw)]
        dac_fs: u2,
        /// RF DC Calibration Count<3:0>
        #[bits(0..=3, rw)]
        rf_dc_calibration_count: u4,
    }
}

pub mod rf_dc_offset_atten {
    use arbitrary_int::u3;
    use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// RF DC Offset Table Update Count<2:0>
        #[bits(5..=7, rw)]
        rf_dc_offset_table_update_count: u3,
        /// RF DC Offset Attenuation<4:0>
        #[bits(0..=4, rw)]
        rf_dc_offset_atten: u5,
    }
}

pub mod invert_bits {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Invert Rx2 RF DC CGin Word
        #[bit(7, rw)]
        invert_rx2_rf_dc_cgin_word: bool,
        /// Invert Rx1 RF DC CGin Word
        #[bit(6, rw)]
        invert_rx1_rf_dc_cgin_word: bool,
        /// Invert Rx2 RF DC CGout Word
        #[bit(5, rw)]
        invert_rx2_rf_dc_cgout_word: bool,
        /// Invert Rx1 RF DC CGout Word
        #[bit(4, rw)]
        invert_rx1_rf_dc_cgout_word: bool,
    }
}

pub mod dc_offset_config2 {
    use arbitrary_int::u3;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Use Wait Counter for RF DC Init Cal
        #[bit(7, rw)]
        use_wait_counter_for_rf_dc_init_cal: bool,
        /// Enable Fast Settle Mode
        #[bit(6, rw)]
        enable_fast_settle_mode: bool,
        /// Enable BB DC Offset Tracking
        #[bit(5, rw)]
        enable_bb_dc_offset_tracking: bool,
        /// Reset Acc on Gain Change
        #[bit(4, rw)]
        reset_acc_on_gain_change: bool,
        /// Enable RF Offset Tracking
        #[bit(3, rw)]
        enable_rf_offset_tracking: bool,
        /// DC Offset Update<2:0>
        #[bits(0..=2, rw)]
        dc_offset_update: u3,
    }
}

pub mod rf_cal_gain_index {
    /// RF Minimum Calibration Gain Index<6:0>
    pub const fn rf_minimum_calibration_gain_index(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod soi_thresh {
    /// RF SOI Threshold<6:0>
    pub const fn rf_soi_thresh(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod bb_dc_offset_shift {
    use arbitrary_int::u2;
    use arbitrary_int::u5;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Increase Count Duration
        #[bit(7, rw)]
        increase_count_duration: bool,
        /// BB Tracking Decimate<1:0>
        #[bits(5..=6, rw)]
        bb_tracking_decimate: u2,
        /// BB DC M Shift<4:0>
        #[bits(0..=4, rw)]
        bb_dc_m_shift: u5,
    }
}

pub mod bb_dc_offset_fast_settle_shift {
    /// Read Back  CH Sel
    pub const READ_BACK_CH_SEL: u8 = 1 << 7;
    /// Update Tracking Word
    pub const UPDATE_TRACKING_WORD: u8 = 1 << 6;
    /// Force Rx Null
    pub const FORCE_RX_NULL: u8 = 1 << 5;

    /// BB DC Tracking Fast Settle M Shift<4:0>
    pub const fn bb_dc_tracking_fast_settle_m_shift(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod bb_dc_offset_count {
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// BB DC Offset Count<5:0>
        #[bits(0..=5, rw)]
        count: u6,
    }
}

pub mod bb_dc_offset_atten {
    use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// BB DC Offset Atten<3:0>
        #[bits(0..=3, rw)]
        bb_dc_offset_atten: u4,
    }
}

pub mod rx1_bb_dc_word_i_msb {
    /// RX1 BB DC Offset Correction word I<14:8>
    pub const fn rx1_bb_dc_offset_correction_word_i(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx1_bb_dc_word_q_msb {
    /// RX1 BB DC Offset Correction word Q<14:8>
    pub const fn rx1_bb_dc_offset_correction_word_q(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx2_bb_dc_word_i_msb {
    /// RX2 BB DC Offset Correction word I<14:8>
    pub const fn rx2_bb_dc_offset_correction_word_i(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx2_bb_dc_word_q_msb {
    /// RX2 BB DC Offset Correction word Q<14:8>
    pub const fn rx2_bb_dc_offset_correction_word_q(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod bb_track_corr_word_i_msb {
    /// RX1/RX2 BB DC Offset Tracking correction word I<14:8>
    pub const fn rx1rx2_bb_dc_offset_tracking_correction_word_i(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod bb_track_corr_word_q_msb {
    /// RX1/RX2 BB DC Offset Tracking correction word Q<14:8>
    pub const fn rx1rx2_bb_dc_offset_tracking_correction_word_q(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod symbol_lsb {
    /// Rx2 RSSI symbol <0>
    pub const RX2_RSSI_SYMBOL: u8 = 1 << 1;
    /// Rx1 RSSI symbol <0>
    pub const RX1_RSSI_SYMBOL: u8 = 1 << 0;
}

pub mod preamble_lsb {
    /// Rx2 RSSI preamble <0>
    pub const RX2_RSSI_PREAMBLE: u8 = 1 << 1;
    /// Rx1 RSSI preamble <0>
    pub const RX1_RSSI_PREAMBLE: u8 = 1 << 0;
}

pub mod rssi_lsb {
    pub const RSSI_LSB_SHIFT: u8 = 1;
    pub const RSSI_LSB_MASK1: u8 = 0x01;
    pub const RSSI_LSB_MASK2: u8 = 0x02;
}

pub mod rx_path_gain_lsb {
    /// Rx Path Gain<0>
    pub const RX_PATH_GAIN: u8 = 1 << 0;
}

pub mod rx_diff_lna_force {
    /// Force Rx2 LNA Gain
    pub const FORCE_RX2_LNA_GAIN: u8 = 1 << 7;
    /// Rx2 LNA Bypass
    pub const RX2_LNA_BYPASS: u8 = 1 << 6;
    /// Force Rx1 LNA Gain
    pub const FORCE_RX1_LNA_GAIN: u8 = 1 << 3;
    /// Rx1 LNA Bypass
    pub const RX1_LNA_BYPASS: u8 = 1 << 2;

    /// Rx2 LNA Gain<1:0>
    pub const fn rx2_lna_gain(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Rx1 LNA Gain<1:0>
    pub const fn rx1_lna_gain(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod rx_lna_bias_coarse {
    /// Rx LNA Bias Coarse<3:0>
    pub const fn rx_lna_bias_coarse(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod rx_lna_bias_fine_0 {
    /// Rx LNA p-Cascode Bias<2:0>
    pub const fn rx_lna_pcascode_bias(x: u8) -> u8 {
        (x & 0x7) << 5
    }

    /// Rx LNA Bias<4:0>
    pub const fn rx_lna_bias(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod rx_lna_bias_fine_1 {
    /// Rx LNA p- Cascode Bias Fine<4:3>
    pub const fn rx_lna_p_cascode_bias_fine(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod rx_mix_gm_config {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(5..=7, rw)]
        rx_mix_gm_cm_out: arbitrary_int::u3,
        #[bits(0..=1, rw)]
        rx_mix_gm_pload: arbitrary_int::u2,
    }
}

pub mod rx1_mix_gm_force {
    /// Force Rx1 Mix Gm
    pub const FORCE_RX1_MIX_GM: u8 = 1 << 6;

    /// Rx1 Mix Gm Gain<5:0>
    pub const fn rx1_mix_gm_gain(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod rx1_mix_gm_bias_force {
    /// Rx1 Mix Gm Bias<4:0>
    pub const fn rx1_mix_gm_bias(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod rx2_mix_gm_force {
    /// Force Rx2 Mix Gm
    pub const FORCE_RX2_MIX_GM: u8 = 1 << 6;

    /// Rx2 Mix Gm Gain<5:0>
    pub const fn rx2_mix_gm_gain(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod rx2_mix_gm_bias_force {
    /// Rx2 Mix Gm Bias<4:0>
    pub const fn rx2_mix_gm_bias(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod input_a_msbs {
    /// Input A RX1 Q<9:8>
    pub const fn input_a_rx1_q(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Input A RX1 I<9:8>
    pub const fn input_a_rx1_i(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Input A RX2 I<9:8>
    pub const fn input_a_rx2_i(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Input A RX2 Q<9:8>
    pub const fn input_a_rx2_q(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod inputs_bc_msbs {
    /// Inputs B&C RX1 Q<9:8>
    pub const fn inputs_bc_rx1_q(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Inputs B&C RX1 I<9:8>
    pub const fn inputs_bc_rx1_i(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Inputs B&C RX2 I<9:8>
    pub const fn inputs_bc_rx2_i(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Inputs B&C RX2 Q<9:8>
    pub const fn inputs_bc_rx2_q(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod force_os_dac {
    /// Force CGin DAC
    pub const FORCE_CGIN_DAC: u8 = 1 << 2;
}

pub mod rx_mix_lo_cm {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(0..=5, rw)]
        rx_mix_lo_cm: arbitrary_int::u6,
    }
}

pub mod rx_cgb_seg_enable {
    /// Rx CGB Seg Enable<5:0>
    pub const fn rx_cgb_seg_enable(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod rx_mix_inputbias {
    /// Rx CGB Input CM Sel<1:0>
    pub const fn rx_cgb_input_cm_sel(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Rx CGB Bias<3:0>
    pub const fn rx_cgb_bias(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod rx_tia_config {
    use arbitrary_int::u3;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// TIA Sel CC<2:0>
        #[bits(5..=7, rw)]
        tia_sel_cc: u3,
        /// TIA2 Override C
        #[bit(3, rw)]
        tia2_override_c: bool,
        /// TIA2 Override R
        #[bit(2, rw)]
        tia2_override_r: bool,
        /// TIA1 Override C
        #[bit(1, rw)]
        tia1_override_c: bool,
        /// TIA1 Override R
        #[bit(0, rw)]
        tia1_override_r: bool,
    }
}

pub mod tia1_c_lsb {
    use arbitrary_int::u2;
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// TIA1 RF<1:0>
        #[bits(6..=7, rw)]
        tia1_rf: u2,
        /// TIA1 C LSB<5:0>
        #[bits(0..=5, rw)]
        tia1_c_lsb: u6,
    }
}

pub mod tia1_c_msb {
    use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// TIA1 C MSB<6:0>
        #[bits(0..=6, rw)]
        tia1_c_msb: u7,
    }
}

pub mod tia2_c_lsb {
    use arbitrary_int::u2;
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// TIA2 RF<1:0>
        #[bits(6..=7, rw)]
        tia2_rf: u2,
        /// TIA2 C LSB<5:0>
        #[bits(0..=5, rw)]
        tia2_c_lsb: u6,
    }
}

pub mod tia2_c_msb {
    use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// TIA2 C MSB<6:0>
        #[bits(0..=6, rw)]
        tia2_c_msb: u7,
    }
}

pub mod rx1_bbf_r1a {
    /// Force Rx1 Resistors
    pub const FORCE_RX1_RESISTORS: u8 = 1 << 7;

    /// Rx1 BBF R1A<5:0>
    pub const fn rx1_bbf_r1a(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod rx2_bbf_r1a {
    /// Force Rx2 Resistors
    pub const FORCE_RX2_RESISTORS: u8 = 1 << 7;

    /// Rx2 BBF R1A<5:0>
    pub const fn rx2_bbf_r1a(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod rx1_tune_ctrl {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(2, rw)]
        rx1_tune_resample_phase: bool,
        #[bit(1, rw)]
        rx1_tune_resample: bool,
        #[bit(0, rw)]
        rx1_pd_tune: bool,
    }
}

pub mod rx2_tune_ctrl {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(2, rw)]
        rx2_tune_resample_phase: bool,
        #[bit(1, rw)]
        rx2_tune_resample: bool,
        #[bit(0, rw)]
        rx2_pd_tune: bool,
    }
}

pub mod rx_bbf_r2346 {
    use arbitrary_int::u3;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tune Override
        #[bit(7, rw)]
        tune_override: bool,
        /// Rx BBF R2346<2:0>
        #[bits(0..=2, rw)]
        rx_bbf_r2346: u3,
    }
}

pub mod rx_bbf_c1_msb {
    /// Rx BBF C1 MSB<5:0>
    pub const fn rx_bbf_c1_msb(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod rx_bbf_c1_lsb {
    /// Rx BBF C1 LSB<6:0>
    pub const fn rx_bbf_c1_lsb(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx_bbf_c2_msb {
    /// Rx BBF C2 MSB<5:0>
    pub const fn rx_bbf_c2_msb(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod rx_bbf_c2_lsb {
    /// Rx BBF C2 LSB<6:0>
    pub const fn rx_bbf_c2_lsb(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx_bbf_c3_msb {
    use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Rx BBF C3 MSB<5:0>
        #[bits(0..=5, rw)]
        rx_bbf_c3_msb: u6,
    }
}

pub mod rx_bbf_c3_lsb {
    use arbitrary_int::u7;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Rx BBF C3 LSB<6:0>
        #[bits(0..=6, rw)]
        rx_bbf_c3_lsb: u7,
    }
}

pub mod rx_bbf_cc1_ctr {
    /// Rx BBF CC1 Ctr<6:0>
    pub const fn rx_bbf_cc1_ctr(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx_bbf_pow_rz_byte0 {
    /// Must be zero
    pub const MUST_BE_ZERO: u8 = 1 << 7;

    /// Rx1 BBF Pow Ctr<1:0>
    pub const fn rx1_bbf_pow_ctr(x: u8) -> u8 {
        (x & 0x3) << 5
    }

    /// Rx BBF Rz1 Ctr<1:0>
    pub const fn rx_bbf_rz1_ctr(x: u8) -> u8 {
        (x & 0x3) << 3
    }
}

pub mod rx_bbf_cc2_ctr {
    /// Rx BBF CC2 Ctr<6:0>
    pub const fn rx_bbf_cc2_ctr(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx_bbf_pow_rz_byte1 {
    /// Rx BBF Pow3 Ctr<1:0>
    pub const fn rx_bbf_pow3_ctr(x: u8) -> u8 {
        (x & 0x3) << 6
    }

    /// Rx BBF RZ3 Ctr<1:0>
    pub const fn rx_bbf_rz3_ctr(x: u8) -> u8 {
        (x & 0x3) << 4
    }

    /// Rx BBF Pow2 Ctr<1:0>
    pub const fn rx_bbf_pow2_ctr(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Rx BBF Rz2 Ctr<1:0>
    pub const fn rx_bbf_rz2_ctr(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod rx_bbf_cc3_ctr {
    /// Rx BBF CC3 Ctr<6:0>
    pub const fn rx_bbf_cc3_ctr(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod rx_bbf_tune {
    /// RxBBF Bypass Bias R
    pub const RXBBF_BYPASS_BIAS_R: u8 = 1 << 7;
    /// Rx BBF R5 Tune
    pub const RX_BBF_R5_TUNE: u8 = 1 << 4;
    /// Rx1 BBF Tune Comp I
    pub const RX1_BBF_TUNE_COMP_I: u8 = 1 << 3;
    /// Rx1 BBF Tune Comp Q
    pub const RX1_BBF_TUNE_COMP_Q: u8 = 1 << 2;
    /// Rx2 BBF Tune Comp I
    pub const RX2_BBF_TUNE_COMP_I: u8 = 1 << 1;
    /// Rx2 BBF Tune Comp Q
    pub const RX2_BBF_TUNE_COMP_Q: u8 = 1 << 0;

    /// Rx BBF Tune Ctr<1:0>
    pub const fn rx_bbf_tune_ctr(x: u8) -> u8 {
        (x & 0x3) << 5
    }
}

pub mod rx1_bbf_man_gain {
    /// Rx1 BBF Force Gain
    pub const RX1_BBF_FORCE_GAIN: u8 = 1 << 5;

    /// Rx1 BBF BQ Gain<1:0>
    pub const fn rx1_bbf_bq_gain(x: u8) -> u8 {
        (x & 0x3) << 3
    }

    /// Rx1 BBF Pole Gain<2:0>
    pub const fn rx1_bbf_pole_gain(x: u8) -> u8 {
        x & 0x7
    }
}

pub mod rx2_bbf_man_gain {
    /// Rx2 BBF Force Gain
    pub const RX2_BBF_FORCE_GAIN: u8 = 1 << 5;

    /// Rx2 BBF BQ Gain<1:0>
    pub const fn rx2_bbf_bq_gain(x: u8) -> u8 {
        (x & 0x3) << 3
    }

    /// Rx2 BBF Pole Gain<2:0>
    pub const fn rx2_bbf_pole_gain(x: u8) -> u8 {
        x & 0x7
    }
}

pub mod rx_bbf_tune_config {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(4, rw)]
        rx_tune_evaltime: bool,
        #[bits(5..=6, rw)]
        tune_comp_mask: arbitrary_int::u2,
        #[bits(1..=3, rw)]
        rx_tune_mode: arbitrary_int::u3,
        #[bit(0, rw)]
        rx_bbf_tune_divide_msb: bool,
    }
}

pub mod pole_gain {
    /// Pole Gain Tune<1:0>
    pub const fn pole_gain_tune(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod rx_bbbw_mhz {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(0..=4, rw)]
        rx_tune_bbbw_mhz: arbitrary_int::u5,
    }
}

pub mod rx_bbbw_khz {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(0..=6, rw)]
        rx_tune_bbbw_khz: arbitrary_int::u7,
    }
}

pub mod rx_pfd_config {
    /// Bypass Ld Synth
    pub const BYPASS_LD_SYNTH: u8 = 1 << 0;
}

pub mod integer_byte_1 {
    pub use arbitrary_int::u3;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Synthesizer Integer Word<10:8>
        #[bits(0..=2, rw)]
        synth_integer_word: u3,
    }
}

pub mod rx_fract_byte_2 {
    /// Synthesizer Fractional Word <22:16>
    pub const fn synth_fract_word(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod force_vco_tune_1 {
    pub use arbitrary_int::{u2, u4};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(3..=6, rw)]
        cal_offset: u4,
        #[bit(1, rw)]
        force_enable: bool,
        #[bit(0, rw)]
        force_tune_upper_bit: bool,
    }
}

pub mod alc_varactor {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Init ALC Value<3:0>
        #[bits(4..=7, rw)]
        init_alc_value: u4,
        /// VCO Varactor<3:0>
        #[bits(0..=3, rw)]
        vco_varactor: u4,
    }
}

pub mod vco_output {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// PORb VCO Logic
        #[bit(6, rw)]
        porb_vco_logic: bool,
        /// VCO Output Level<3:0>
        #[bits(0..=3, rw)]
        vco_output_level: u4,
    }
}

pub mod cp_current {
    pub use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Charge Pump Current<5:0>
        #[bits(0..=5, rw)]
        charge_pump_current: u6,
    }
}

pub mod cp_offset {
    pub use arbitrary_int::u6;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Synth Re-Cal
        #[bit(7, rw)]
        synth_recal: bool,
        /// Charge Pump Offset<5:0>
        #[bits(0..=5, rw)]
        charge_pump_offset: u6,
    }
}

pub mod rx_cp_config {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Half Vco Cal Clk
        #[bit(7, rw)]
        half_vco_cal_clk: bool,
        /// CP Offset Off
        #[bit(4, rw)]
        cp_offset_off: bool,
        /// F Cpcal
        #[bit(3, rw)]
        f_cpcal: bool,
        /// Cp Cal Enable
        #[bit(2, rw)]
        cp_cal_enable: bool,
    }
}

pub mod loop_filter_1 {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Loop Filter C2<3:0>
        #[bits(4..=7, rw)]
        loop_filter_c2: u4,
        /// Loop Filter C1<3:0>
        #[bits(0..=3, rw)]
        loop_filter_c1: u4,
    }
}

pub mod loop_filter_2 {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Loop Filter R1<3:0>
        #[bits(4..=7, rw)]
        loop_filter_r1: u4,
        /// Loop Filter C3<3:0>
        #[bits(0..=3, rw)]
        loop_filter_c3: u4,
    }
}

pub mod loop_filter_3 {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Loop Filter Bypass R3
        #[bit(7, rw)]
        loop_filter_bypass_r3: bool,
        /// Loop Filter Bypass R1
        #[bit(6, rw)]
        loop_filter_bypass_r1: bool,
        /// Loop Filter Bypass C2
        #[bit(5, rw)]
        loop_filter_bypass_c2: bool,
        /// Loop Filter Bypass C1
        #[bit(4, rw)]
        loop_filter_bypass_c1: bool,
        /// Loop Filter R3<3:0>
        #[bits(0..=3, rw)]
        loop_filter_r3: u4,
    }
}

pub mod rx_dithercp_cal {
    /// Forced CP Cal Word<3:0>
    pub const fn forced_cp_cal_word(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod rx_vco_bias_1 {
    pub use arbitrary_int::{u2, u3};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// VCO Bias Tcf<1:0>
        #[bits(3..=4, rw)]
        vco_bias_tcf: u2,
        /// VCO Bias Ref<2:0>
        #[bits(0..=2, rw)]
        vco_bias_ref: u3,
    }
}

pub mod rx_cal_status {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// CP Cal Valid
        #[bit(7, r)]
        cp_cal_valid: bool,
        /// CP Cal Done
        #[bit(5, r)]
        cp_cal_done: bool,
        /// VCO Cal Busy
        #[bit(4, r)]
        vco_cal_busy: bool,
        /// CP Cal Word<3:0>
        #[bits(0..=3, r)]
        cp_cal_word: u4,
    }
}

pub mod vco_cal_ref {
    pub use arbitrary_int::u3;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// VCO Cal Ref Tcf<2:0>
        #[bits(0..=2, rw)]
        vco_cal_ref_tcf: u3,
    }
}

pub mod rx_vco_pd_overrides {
    /// Power Down Varactor Ref
    pub const POWER_DOWN_VARACTOR_REF: u8 = 1 << 3;
    /// Pwr Down Varact Ref Tcf
    pub const PWR_DOWN_VARACT_REF_TCF: u8 = 1 << 2;
    /// Power Down Cal Tcf
    pub const POWER_DOWN_CAL_TCF: u8 = 1 << 1;
    /// Power Down VCO Bufffer
    pub const POWER_DOWN_VCO_BUFFFER: u8 = 1 << 0;
}

pub mod cp_overrange_vco_lock {
    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// CP Ovrg High
        #[bit(7, rw)]
        cp_ovrg_high: bool,
        /// CP Ovrg Low
        #[bit(6, rw)]
        cp_ovrg_low: bool,
        /// Lock
        #[bit(1, rw)]
        vco_lock: bool,
    }
}

pub mod rx_vco_ldo {
    /// VCO LDO Bypass
    pub const VCO_LDO_BYPASS: u8 = 1 << 7;

    /// VCO LDO Inrush<1:0>
    pub const fn vco_ldo_inrush(x: u8) -> u8 {
        (x & 0x3) << 5
    }

    /// VCO LDO Sel<2:0>
    pub const fn vco_ldo_sel(x: u8) -> u8 {
        (x & 0x7) << 2
    }

    /// VCO LDO Vdrop Sel<1:0>
    pub const fn vco_ldo_vdrop_sel(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod rx_vco_cal {
    pub use arbitrary_int::{u2, u3};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// VCO Cal En
        #[bit(7, rw)]
        vco_cal_en: bool,
        /// VCO Cal ALC Wait<2:0>
        #[bits(4..=6, rw)]
        vco_cal_alc_wait: u3,
        /// VCO Cal Count<1:0>
        #[bits(2..=3, rw)]
        vco_cal_count: u2,
        /// Must be set to 0b10
        #[bits(0..=1, rw)]
        should_be_0b10: u2,
    }
}

pub mod rx_lock_detect_config {
    /// Lock Detect Count<1:0>
    pub const fn lock_detect_count(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Lock Detect Mode<1:0>
    pub const fn lock_detect_mode(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod rx_cp_level_detect {
    /// CP Level Detect Power Down
    pub const CP_LEVEL_DETECT_POWER_DOWN: u8 = 1 << 6;

    /// CP Level Threshold Low<2:0>
    pub const fn cp_level_thresh_low(x: u8) -> u8 {
        (x & 0x7) << 3
    }

    /// CP Level Threshold High<2:0>
    pub const fn cp_level_thresh_high(x: u8) -> u8 {
        x & 0x7
    }
}

pub mod rx_dsm_setup_0 {
    /// DSM Prog<3:0>
    pub const fn dsm_prog(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod rx_dsm_setup_1 {
    /// SIF clock
    pub const SIF_CLOCK: u8 = 1 << 6;
    /// SIF Reset Bar
    pub const SIF_RESET_BAR: u8 = 1 << 5;

    /// SIF Addr<4:0>
    pub const fn sif_addr(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod rx_correction_word0 {
    /// Update Freq Word
    pub const UPDATE_FREQ_WORD: u8 = 1 << 7;
    /// Read Effective Tuning Word
    pub const READ_EFFECTIVE_TUNING_WORD: u8 = 1 << 5;

    /// Frequency Correction Word<11:7>
    pub const fn freq_correction_word_msb(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod rx_correction_word1 {
    /// Update Freq Word
    pub const UPDATE_FREQ_WORD: u8 = 1 << 7;

    /// Frequency Correction Word<6:0>
    pub const fn freq_correction_word_lsb(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod vco_varactor_ctrl_0 {
    pub use arbitrary_int::{u3, u4};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// VCO Varactor Reference Tcf<2:0>
        #[bits(4..=6, rw)]
        vco_varactor_reference_tcf: u3,
        /// VCO Varactor Offset<3:0>
        #[bits(0..=3, rw)]
        vco_varactor_offset: u4,
    }
}

pub mod vco_varactor_ctrl_1 {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// VCO Varactor Reference<3:0>
        #[bits(0..=3, rw)]
        vco_varactor_reference: u4,
    }
}

pub mod rx_fast_lock_setup {
    /// Rx Fast Lock Load Synth
    pub const RX_FAST_LOCK_LOAD_SYNTH: u8 = 1 << 3;
    /// Rx Fast Lock Profile Init
    pub const RX_FAST_LOCK_PROFILE_INIT: u8 = 1 << 2;
    /// Rx Fast Lock Profile Pin Select
    pub const RX_FAST_LOCK_PROFILE_PIN_SELECT: u8 = 1 << 1;
    /// Rx Fast Lock Mode Enable
    pub const RX_FAST_LOCK_MODE_ENABLE: u8 = 1 << 0;

    /// Rx Fast Lock Profile<2:0>
    pub const fn rx_fast_lock_profile(x: u8) -> u8 {
        (x & 0x7) << 5
    }
}

pub mod rx_fast_lock_program_addr {
    /// Rx Fast Lock Profile<2:0>
    pub const fn rx_fast_lock_profile_addr(x: u8) -> u8 {
        (x & 0x7) << 4
    }

    /// Configuration Word <3:0>
    pub const fn rx_fast_lock_profile_word(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod rx_fast_lock_program_ctrl {
    /// Rx Fast Lock Program Write
    pub const RX_FAST_LOCK_PROGRAM_WRITE: u8 = 1 << 1;
    /// Rx Fast Lock Program Clock Enable
    pub const RX_FAST_LOCK_PROGRAM_CLOCK_ENABLE: u8 = 1 << 0;

    pub const RX_FAST_LOCK_CONFIG_WORD_NUM: usize = 16;
}

pub mod rx_lo_gen_power_mode {
    /// Power Mode<3:0>
    pub const fn rx_lo_gen_power_mode(x: u8) -> u8 {
        (x & 0x3) << 4
    }
}

pub mod tx_pfd_config {
    /// Div Test En
    pub const DIV_TEST_EN: u8 = 1 << 5;
    /// PFD Clk Edge
    pub const PFD_CLK_EDGE: u8 = 1 << 1;
    /// Bypass Ld Synth
    pub const BYPASS_LD_SYNTH: u8 = 1 << 0;

    /// PFD Width <1:0>
    pub const fn pfd_width(x: u8) -> u8 {
        (x & 0x3) << 2
    }
}

pub mod tx_fract_byte_2 {
    /// Synthesizer Fractional Word <22:16>
    pub const fn synth_fract_word(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod tx_force_alc {
    /// Force ALC Enable
    pub const FORCE_ALC_ENABLE: u8 = 1 << 7;

    /// Force ALC Word<6:0>
    pub const fn force_alc_word(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod tx_alcvaract_or {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Init ALC Value<3:0>
        #[bits(4..=7, rw)]
        init_alc_value: u4,
        /// VCO Varactor<3:0>
        #[bits(0..=3, rw)]
        vco_varactor: u4,
    }
}

pub mod tx_vco_output {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// PORb VCO Logic
        #[bit(6, rw)]
        porb_vco_logic: bool,
        /// VCO Output Level<3:0>
        #[bits(0..=3, rw)]
        vco_output_level: u4,
    }
}

pub mod tx_cp_config {
    pub use arbitrary_int::u2;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Half Vco Cal Clk
        #[bit(7, rw)]
        half_vco_cal_clk: bool,
        /// Dither Mode
        #[bit(6, rw)]
        dither_mode: bool,
        /// Cp Offset Off
        #[bit(4, rw)]
        cp_offset_off: bool,
        /// F Cpcal
        #[bit(3, rw)]
        f_cpcal: bool,
        /// Cp Cal Enable
        #[bit(2, rw)]
        cp_cal_enable: bool,
        /// Cp Test<1:0>
        #[bits(0..=1, rw)]
        cp_test: u2,
    }
}

pub mod tx_dithercp_cal {
    /// Number SDM Dither Bits<3:0>
    pub const fn number_sdm_dither_bits(x: u8) -> u8 {
        (x & 0xF) << 4
    }

    /// Forced CP Cal Word<3:0>
    pub const fn forced_cp_cal_word(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod vco_bias_1 {
    pub use arbitrary_int::{u2, u3};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Must be zeros
        #[bits(5..=6, rw)]
        must_be_zeros: u2,
        /// VCO Bias Tcf<1:0>
        #[bits(3..=4, rw)]
        vco_bias_tcf: u2,
        /// VCO Bias Ref<2:0>
        #[bits(0..=2, rw)]
        vco_bias_ref: u3,
    }
}

pub mod tx_vco_bias_2 {
    /// VCO Bypass Bias DAC R
    pub const VCO_BYPASS_BIAS_DAC_R: u8 = 1 << 7;
    /// VCO Cop Bypass Bias R
    pub const VCO_COMP_BYPASS_BIAS_R: u8 = 1 << 4;
    /// Bypass Prescale R
    pub const BYPASS_PRESCALE_R: u8 = 1 << 3;
    /// Last ALC Enable
    pub const LAST_ALC_ENABLE: u8 = 1 << 2;

    /// Prescale Bias <1:0>
    pub const fn prescale_bias(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod tx_cal_status {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// CP Cal Valid
        #[bit(7, r)]
        cp_cal_valid: bool,
        /// Comp Out
        #[bit(6, r)]
        comp_out: bool,
        /// CP Cal Done
        #[bit(5, r)]
        cp_cal_done: bool,
        /// VCO Cal Busy
        #[bit(4, r)]
        vco_cal_busy: bool,
        /// CP Cal Word<3:0>
        #[bits(0..=3, r)]
        cp_cal_word: u4,
    }
}

pub mod tx_vco_pd_overrides {
    /// Power Down Varactor Ref
    pub const POWER_DOWN_VARACTOR_REF: u8 = 1 << 3;
    /// Power Down Varact Ref Tcf
    pub const POWER_DOWN_VARACT_REF_TCF: u8 = 1 << 2;
    /// Power Down Cal Tcf
    pub const POWER_DOWN_CAL_TCF: u8 = 1 << 1;
    /// Power Down VCO Bufffer
    pub const POWER_DOWN_VCO_BUFFFER: u8 = 1 << 0;
}

pub mod tx_vco_ldo {
    /// VCO LDO Bypass
    pub const VCO_LDO_BYPASS: u8 = 1 << 7;

    /// VCO LDO Inrush<1:0>
    pub const fn vco_ldo_inrush(x: u8) -> u8 {
        (x & 0x3) << 5
    }

    /// VCO LDO Vout Sel<2:0>
    pub const fn vco_ldo_vout_sel(x: u8) -> u8 {
        (x & 0x7) << 2
    }

    /// VCO LDO Vdrop Sel<1:0>
    pub const fn vco_ldo_vdrop_sel(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod tx_vco_cal {
    pub use arbitrary_int::{u2, u3};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// VCO Cal En
        #[bit(7, rw)]
        vco_cal_en: bool,
        /// VCO Cal ALC Wait<2:0>
        #[bits(4..=6, rw)]
        vco_cal_alc_wait: u3,
        /// VCO Cal Count<1:0>
        #[bits(2..=3, rw)]
        vco_cal_count: u2,
        /// FB Clock Adv<1:0>
        #[bits(0..=1, rw)]
        fb_clock_adv: u2,
    }
}

pub mod tx_lock_detect_config {
    /// Lock Detect Count<1:0>
    pub const fn lock_detect_count(x: u8) -> u8 {
        (x & 0x3) << 2
    }

    /// Lock Detect Mode<1:0>
    pub const fn lock_detect_mode(x: u8) -> u8 {
        x & 0x3
    }
}

pub mod tx_cp_level_detect {
    /// CP Level Detect Power Down
    pub const CP_LEVEL_DETECT_POWER_DOWN: u8 = 1 << 6;

    /// CP Level Detect Threshold Low<2:0>
    pub const fn cp_level_detect_thresh_low(x: u8) -> u8 {
        (x & 0x7) << 3
    }

    /// CP Level Detect Threshold High<2:0>
    pub const fn cp_level_detect_thresh_high(x: u8) -> u8 {
        x & 0x7
    }
}

pub mod tx_dsm_setup_0 {
    /// DSM Prog<3:0>
    pub const fn dsm_prog(x: u8) -> u8 {
        x & 0xF
    }
}

pub mod tx_dsm_setup_1 {
    /// SIF clock
    pub const SIF_CLOCK: u8 = 1 << 6;
    /// SIF Reset Bar
    pub const SIF_RESET_BAR: u8 = 1 << 5;

    /// SIF Addr<4:0>
    pub const fn sif_addr(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod tx_correction_word0 {
    /// Update Freq Word
    pub const UPDATE_FREQ_WORD: u8 = 1 << 7;
    /// Read Effective Tuning Word
    pub const READ_EFFECTIVE_TUNING_WORD: u8 = 1 << 5;

    /// Frequency Correction Word<11:7>
    pub const fn freq_correction_word_msb(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod tx_correction_word1 {
    /// Update Freq Word
    pub const UPDATE_FREQ_WORD: u8 = 1 << 7;

    /// Frequency Correction Word<6:0>
    pub const fn freq_correction_word_lsb(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod dcxo_coarse_tune {
    /// DCXO Tune Coarse<5:0>
    pub const fn dcxo_tune_coarse(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod dcxo_fine_tune_low {
    /// DCXO Tune Fine<4:0>
    pub const fn dcxo_tune_fine_low(x: u8) -> u8 {
        (x & 0x1F) << 3
    }
}

pub mod dcxo_fine_tune_high {
    /// DCXO Tune Fine<12:5>
    pub const fn dcxo_tune_fine_high(x: u16) -> u16 {
        x >> 5
    }
}

pub mod dcxo_config {
    /// Must be zero
    pub const MUST_BE_ZERO: u8 = 1 << 7;

    /// DCXO Rtail<2:0>
    pub const fn dcxo_rtail(x: u8) -> u8 {
        (x & 0x7) << 4
    }

    /// DCXO Rd<1:0>
    pub const fn dcxo_rd(x: u8) -> u8 {
        (x & 0x3) << 2
    }
}

pub mod dcxo_tempco_addr {
    /// DCXO Tempco En
    pub const DCXO_TEMPCO_EN: u8 = 1 << 7;
    /// DCXO Tempco Clk
    pub const DCXO_TEMPCO_CLK: u8 = 1 << 6;

    /// DCXO Temperature Coefficient Address<5:0>
    pub const fn dcxo_temperature_coef_address(x: u8) -> u8 {
        x & 0x3F
    }
}

pub mod tx_fast_lock_setup {
    /// Tx Fast Lock Load Synth
    pub const TX_FAST_LOCK_LOAD_SYNTH: u8 = 1 << 3;
    /// Tx Fast Lock Profile Init
    pub const TX_FAST_LOCK_PROFILE_INIT: u8 = 1 << 2;
    /// Tx Fast Lock Profile Pin Select
    pub const TX_FAST_LOCK_PROFILE_PIN_SELECT: u8 = 1 << 1;
    /// Tx Fast Lock Mode Enable
    pub const TX_FAST_LOCK_MODE_ENABLE: u8 = 1 << 0;

    /// Tx Fast Lock Profile<2:0>
    pub const fn tx_fast_lock_profile(x: u8) -> u8 {
        (x & 0x7) << 5
    }
}

pub mod tx_fast_lock_program_ctrl {
    /// Tx Fast Lock Program Write
    pub const TX_FAST_LOCK_PROGRAM_WRITE: u8 = 1 << 1;
    /// Tx Fast Lock Program Clock Enable
    pub const TX_FAST_LOCK_PROGRAM_CLOCK_ENABLE: u8 = 1 << 0;
}

pub mod tx_lo_gen_power_mode {
    /// Power Mode<3:0>
    pub const fn tx_lo_gen_power_mode(x: u8) -> u8 {
        (x & 0xF) << 4
    }
}

pub mod bandgap_config0 {
    /// Power Down Bandgap Ref
    pub const POWER_DOWN_BANDGAP_REF: u8 = 1 << 7;
    /// Master Bias Filter Bypass
    pub const MASTER_BIAS_FILTER_BYPASS: u8 = 1 << 6;
    /// Master Bias Ref Sel
    pub const MASTER_BIAS_REF_SEL: u8 = 1 << 5;

    /// Master Bias Trim<4:0>
    pub const fn master_bias_trim(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod bandgap_config1 {
    /// VCO LDO Filter Bypass
    pub const VCO_LDO_FILTER_BYPASS: u8 = 1 << 7;
    /// VCO LDO Ref Sel
    pub const VCO_LDO_REF_SEL: u8 = 1 << 6;
    /// Bandgap Ref Reset
    pub const BANDGAP_REF_RESET: u8 = 1 << 5;

    /// Bandgap Temp Trim<4:0>
    pub const fn bandgap_temp_trim(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod ref_divide_config_1 {
    use arbitrary_int::u2;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bits(1..=2, rw)]
        should_be_ones: u2,
        #[bit(0, rw)]
        rx_ref_divider_upper_bit: bool,
    }
}

pub mod ref_divide_config_2 {
    use arbitrary_int::{u2, u3};

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        #[bit(7, rw)]
        rx_ref_divider_lower_bit: bool,
        #[bits(4..=6, rw)]
        should_be_ones_1: u3,
        #[bits(2..=3, rw)]
        tx_ref_divider: super::ClockScaler,
        #[bits(0..=1, rw)]
        should_be_ones_0: u2,
    }
}

pub mod gain_rx {
    /// Full Table Gain Index Rx1/LMT Gain Rx1<6:0>
    pub const fn full_table_gain_index(x: u8) -> u8 {
        x & 0x7F
    }
}

pub mod lpf_gain_rx {
    /// LPF gain Rx1<4:0>
    pub const fn lpf_gain_rx(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod dig_gain_rx {
    /// Digital gain Rx1<4:0>
    pub const fn digital_gain_rx(x: u8) -> u8 {
        x & 0x1F
    }
}

pub mod fast_attack_state {
    /// Fast Attack State Rx2<2:0>
    pub const fn fast_attack_state_rx2(x: u8) -> u8 {
        (x & 0x7) << 4
    }

    /// Fast Attack State Rx1<2:0>
    pub const fn fast_attack_state_rx1(x: u8) -> u8 {
        x & 0x7
    }

    pub const FAST_ATK_MASK: u8 = 0x7;
    pub const RX1_FAST_ATK_SHIFT: u8 = 0;
    pub const RX2_FAST_ATK_SHIFT: u8 = 4;
    pub const FAST_ATK_RESET: u8 = 0;
    pub const FAST_ATK_PEAK_DETECT: u8 = 1;
    pub const FAST_ATK_PWR_MEASURE: u8 = 2;
    pub const FAST_ATK_FINAL_SETTELING: u8 = 3;
    pub const FAST_ATK_FINAL_OVER: u8 = 4;
    pub const FAST_ATK_GAIN_LOCKED: u8 = 5;
}

pub mod slow_loop_state {
    /// Slow Loop State Rx2<2:0>
    pub const fn slow_loop_state_rx2(x: u8) -> u8 {
        (x & 0x7) << 4
    }

    /// Slow Loop State Rx1<2:0>
    pub const fn slow_loop_state_rx1(x: u8) -> u8 {
        x & 0x7
    }
}

pub mod ovrg_sigs_rx {
    /// Gain Lock 1
    pub const GAIN_LOCK_1: u8 = 1 << 6;
    /// Low Power 1
    pub const LOW_POWER_1: u8 = 1 << 5;
    /// Large LMT OL
    pub const LARGE_LMT_OL: u8 = 1 << 4;
    /// Small LMT OL
    pub const SMALL_LMT_OL: u8 = 1 << 3;
    /// Large ADC OL
    pub const LARGE_ADC_OL: u8 = 1 << 2;
    /// Small ADC OL
    pub const SMALL_ADC_OL: u8 = 1 << 1;
    /// Dig Sat
    pub const DIG_SAT: u8 = 1 << 0;
}

pub mod ctrl {
    /// Set to 1
    pub const CTRL_ENABLE: u8 = 1 << 0;
}

pub mod bist_config {
    pub use arbitrary_int::u2;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Tone Frequency<1:0>
        #[bits(6..=7, rw)]
        tone_freq: u2,
        /// Tone Level<1:0>
        #[bits(4..=5, rw)]
        tone_level: u2,
        /// BIST Control Point<1:0>. `0` = TX injection, `2` = RX injection (ADC digital
        /// output); `1`/`3` are not used by this driver and their meaning isn't verified here.
        #[bits(2..=3, rw)]
        bist_ctrl_point: u2,
        /// Tone/PRBS
        #[bit(1, rw)]
        tone_prbs: bool,
        /// BIST Enable
        #[bit(0, rw)]
        bist_enable: bool,
    }
}

pub mod bist_config_2 {
    pub use arbitrary_int::u4;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Data Port SP, HD Loop Test OE
        #[bit(7, rw)]
        data_port_sp_hd_loop_test_oe: bool,
        /// Rx Mask
        #[bit(6, rw)]
        rx_mask: bool,
        /// Channel
        #[bit(5, rw)]
        channel: bool,
        /// Observation Point<2:0>
        #[bits(1..=4, rw)]
        observation_point: u4,
        /// Data Port Loop Test Enable
        #[bit(0, rw)]
        data_port_loop_test_enable: bool,
    }
}

pub mod bist_and_data_port_test_config {
    pub use arbitrary_int::u2;

    #[bitbybit::bitfield(
        u8,
        default = 0,
        debug,
        defmt_bitfields(feature = "defmt"),
        forbid_overlaps
    )]
    pub struct Register {
        /// Temp Sense Vbe Test<1:0>
        #[bits(6..=7, rw)]
        temp_sense_vbe_test: u2,
        /// BIST Mask Channel 2 Q data
        #[bit(5, rw)]
        bist_mask_channel_2_q_data: bool,
        /// BIST Mask Channel 2 I data
        #[bit(4, rw)]
        bist_mask_channel_2_i_data: bool,
        /// BIST Mask Channel 1 Q data
        #[bit(3, rw)]
        bist_mask_channel_1_q_data: bool,
        /// BIST Mask Channel 1 I data
        #[bit(2, rw)]
        bist_mask_channel_1_i_data: bool,
        /// Data Port Hi/Low
        #[bit(1, rw)]
        data_port_hilow: bool,
        /// Use Data Port
        #[bit(0, rw)]
        use_data_port: bool,
    }
}

pub mod dac_test_2 {
    /// DAC Test Enable
    pub const DAC_TEST_ENABLE: u8 = 1 << 7;

    /// DAC test Word <22:16>
    pub const fn dac_test_word(x: u8) -> u8 {
        x & 0x7F
    }
}
