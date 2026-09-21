use arbitrary_int::{u3, u10};

pub const MODE: embedded_hal::spi::Mode = embedded_hal::spi::MODE_1;

pub const AD_READ: u16 = 0 << 15;
pub const AD_WRITE: u16 = 1 << 15;

#[bitbybit::bitfield(u16, default = 0x0, debug)]
pub struct SpiHeader {
    #[bit(15, rw)]
    write_bit: bool,
    /// Value + 1 is the number of bytes transferred.
    #[bits(12..=14, rw)]
    count: u3,
    #[bits(0..=9, rw)]
    addr: u10,
}

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
