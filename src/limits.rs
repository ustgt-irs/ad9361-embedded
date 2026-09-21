pub const MIN_RF_BW: u32 = 200_000;
pub const MAX_RF_BW: u32 = 56_000_000;

pub const RSSI_MULTIPLIER: i32 = 100;
pub const RSSI_RESOLUTION: i32 = 25;
pub const RSSI_MAX_WEIGHT: u8 = 255;

pub const MAX_LMT_INDEX: u8 = 40;
pub const MAX_LPF_GAIN: u8 = 24;
pub const MAX_DIG_GAIN: u8 = 31;

/// 70 MHz + 100ppm
pub const MAX_BBPLL_FREF: u32 = 70_007_000;
/// 715 MHz - 100ppm
pub const MIN_BBPLL_FREQ: u32 = 714_928_500;
/// 1430 MHz + 100ppm
pub const MAX_BBPLL_FREQ: u32 = 1_430_143_000;
pub const MAX_BBPLL_DIV: u8 = 64;
pub const MIN_BBPLL_DIV: u8 = 2;

/// 25 MHz
pub const MIN_ADC_CLK: u32 = 25_000_000;
/// 640 MHz
pub const MAX_ADC_CLK: u32 = 640_000_000;
pub const MAX_DAC_CLK: u32 = MAX_ADC_CLK / 2;

/// Associated with outputs of stage
pub const MAX_RX_HB1: u32 = 245_760_000;
pub const MAX_RX_HB2: u32 = 320_000_000;
pub const MAX_RX_HB3: u32 = 640_000_000;

/// Associated with inputs of stage
pub const MAX_TX_HB1: u32 = 160_000_000;
pub const MAX_TX_HB2: u32 = 320_000_000;
pub const MAX_TX_HB3: u32 = 320_000_000;

pub const MAX_BASEBAND_RATE: u32 = 61_440_000;

pub const MAX_MBYTE_SPI: u8 = 8;

pub const RFPLL_MODULUS: u32 = 8_388_593;
pub const BBPLL_MODULUS: u32 = 2_088_960;

/// 80 MHz + 100ppm
pub const MAX_SYNTH_FREF: u32 = 80_008_000;
/// 10 MHz - 100ppm
pub const MIN_SYNTH_FREF: u32 = 9_999_000;

pub const MIN_VCO_FREQ_HZ: u64 = 6_000_000_000;
pub const MAX_VCO_FREQ_HZ: u64 = 12_000_000_000;

pub const MAX_CARRIER_FREQ_HZ: u64 = 6_000_000_000;
pub const MIN_RX_CARRIER_FREQ_HZ: u64 = 70_000_000;
pub const MIN_TX_CARRIER_FREQ_HZ: u64 = 46_875_001;

pub const AD9363A_MAX_CARRIER_FREQ_HZ: u64 = 3_800_000_000;
pub const AD9363A_MIN_CARRIER_FREQ_HZ: u64 = 325_000_000;

pub const MAX_TX_ATTENUATION_DB: u32 = 89_750;
