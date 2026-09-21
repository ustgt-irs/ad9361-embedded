/// Minimum RF bandwidth in Hz.
pub const MIN_RF_BW: u32 = 200_000;
/// Maximum RF bandwidth in Hz.
pub const MAX_RF_BW: u32 = 56_000_000;

/// Fixed-point scale of RSSI values. An RSSI value divided by this is in dB.
pub const RSSI_MULTIPLIER: i32 = 100;
/// RSSI resolution of 0.25 dB, in units of `1 / RSSI_MULTIPLIER` dB.
pub const RSSI_RESOLUTION: i32 = 25;
/// The RSSI weights of all measurement durations add up to this value.
pub const RSSI_MAX_WEIGHT: u8 = 255;

/// Maximum LMT index of a manual RX gain setting.
pub const MAX_LMT_INDEX: u8 = 40;
/// Maximum LPF gain of a manual RX gain setting.
pub const MAX_LPF_GAIN: u8 = 24;
/// Maximum digital gain of a manual RX gain setting.
pub const MAX_DIG_GAIN: u8 = 31;

/// 70 MHz + 100ppm
pub const MAX_BBPLL_FREF: u32 = 70_007_000;
/// 715 MHz - 100ppm
pub const MIN_BBPLL_FREQ: u32 = 714_928_500;
/// 1430 MHz + 100ppm
pub const MAX_BBPLL_FREQ: u32 = 1_430_143_000;
/// Maximum BB PLL divider.
pub const MAX_BBPLL_DIV: u8 = 64;
/// Minimum BB PLL divider.
pub const MIN_BBPLL_DIV: u8 = 2;

/// 25 MHz
pub const MIN_ADC_CLK: u32 = 25_000_000;
/// 640 MHz
pub const MAX_ADC_CLK: u32 = 640_000_000;
/// Maximum DAC clock in Hz, half of the maximum ADC clock.
pub const MAX_DAC_CLK: u32 = MAX_ADC_CLK / 2;

/// Maximum output clock of the RX HB1 stage in Hz.
pub const MAX_RX_HB1: u32 = 245_760_000;
/// Maximum output clock of the RX HB2 stage in Hz.
pub const MAX_RX_HB2: u32 = 320_000_000;
/// Maximum output clock of the RX HB3 stage in Hz.
pub const MAX_RX_HB3: u32 = 640_000_000;

/// Maximum input clock of the TX HB1 stage in Hz.
pub const MAX_TX_HB1: u32 = 160_000_000;
/// Maximum input clock of the TX HB2 stage in Hz.
pub const MAX_TX_HB2: u32 = 320_000_000;
/// Maximum input clock of the TX HB3 stage in Hz.
pub const MAX_TX_HB3: u32 = 320_000_000;

/// Maximum baseband sample rate in Hz.
pub const MAX_BASEBAND_RATE: u32 = 61_440_000;

/// Maximum number of bytes in one multi-byte SPI transfer.
pub const MAX_MBYTE_SPI: u8 = 8;

/// Modulus of the fractional RF PLL divider.
pub const RFPLL_MODULUS: u32 = 8_388_593;
/// Modulus of the fractional BB PLL divider.
pub const BBPLL_MODULUS: u32 = 2_088_960;

/// 80 MHz + 100ppm
pub const MAX_SYNTH_FREF: u32 = 80_008_000;
/// 10 MHz - 100ppm
pub const MIN_SYNTH_FREF: u32 = 9_999_000;

/// Minimum RF VCO frequency in Hz.
pub const MIN_VCO_FREQ_HZ: u64 = 6_000_000_000;
/// Maximum RF VCO frequency in Hz.
pub const MAX_VCO_FREQ_HZ: u64 = 12_000_000_000;

/// Maximum RX and TX carrier frequency in Hz.
pub const MAX_CARRIER_FREQ_HZ: u64 = 6_000_000_000;
/// Minimum RX carrier frequency in Hz.
pub const MIN_RX_CARRIER_FREQ_HZ: u64 = 70_000_000;
/// Minimum TX carrier frequency in Hz.
pub const MIN_TX_CARRIER_FREQ_HZ: u64 = 46_875_001;

/// Maximum carrier frequency of the AD9363A in Hz.
pub const AD9363A_MAX_CARRIER_FREQ_HZ: u64 = 3_800_000_000;
/// Minimum carrier frequency of the AD9363A in Hz.
pub const AD9363A_MIN_CARRIER_FREQ_HZ: u64 = 325_000_000;

/// Maximum TX attenuation in milli-dB, which is 89.75 dB.
pub const MAX_TX_ATTENUATION_DB: u32 = 89_750;
