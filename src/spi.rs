use arbitrary_int::{u3, u10};

/// SPI mode required by the AD9361.
pub const MODE: embedded_hal::spi::Mode = embedded_hal::spi::MODE_1;

/// Read bit of the SPI header.
pub const AD_READ: u16 = 0 << 15;
/// Write bit of the SPI header.
pub const AD_WRITE: u16 = 1 << 15;

/// Two byte header in front of every SPI transfer.
#[bitbybit::bitfield(u16, default = 0x0, debug)]
pub struct SpiHeader {
    /// Set for a write, cleared for a read.
    #[bit(15, rw)]
    write_bit: bool,
    /// Value + 1 is the number of bytes transferred.
    #[bits(12..=14, rw)]
    count: u3,
    /// Address of the first register.
    #[bits(0..=9, rw)]
    addr: u10,
}

/// Writes the header of a read of `length` bytes starting at `reg` to the start of `buf`.
///
/// Returns the payload part of `buf`. It is empty and `buf` is untouched if `length` is 0.
pub fn prepare_spi_read(buf: &mut [u8], reg: u10, length: u3) -> &mut [u8] {
    if length.value() == 0 {
        return &mut [];
    }
    let header = SpiHeader::builder()
        .with_write_bit(false)
        .with_count(length.saturating_sub(u3::new(1)))
        .with_addr(reg)
        .build()
        .raw_value();
    buf[0..2].copy_from_slice(&header.to_be_bytes());
    &mut buf[2..2 + length.value() as usize]
}

/// Writes the header of a write of `length` bytes starting at `reg` to the start of `buf`.
///
/// Returns the payload part of `buf`. It is empty and `buf` is untouched if `length` is 0.
pub fn prepare_spi_write(buf: &mut [u8], reg: u10, length: u3) -> &mut [u8] {
    if length.value() == 0 {
        return &mut [];
    }
    let header = SpiHeader::builder()
        .with_write_bit(true)
        .with_count(length.saturating_sub(u3::new(1)))
        .with_addr(reg)
        .build()
        .raw_value();
    buf[0..2].copy_from_slice(&header.to_be_bytes());
    &mut buf[2..2 + length.value() as usize]
}
