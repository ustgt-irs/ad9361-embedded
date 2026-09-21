//! In-memory AD9361 model behind the SPI, GPIO and delay traits.
use std::convert::Infallible;

use ad9361_embedded::{regs::Register, spi::AD_WRITE};
use embedded_hal::spi::{ErrorType, Operation};

const REG_COUNT: usize = 1 << 10;
/// Product ID 1 (AD9361) in bits 7:3, silicon revision 2 in bits 2:0.
const PRODUCT_ID_AD9361_REV2: u8 = 0x0A;

const PRODUCT_ID: usize = Register::ProductId as usize;
const CALIBRATION_CTRL: usize = Register::CalibrationCtrl as usize;
const STATE: usize = Register::State as usize;
const ENSM_CONFIG_1: usize = Register::EnsmConfig1 as usize;
/// Shares the register with the overflow flags.
const BB_PLL_LOCK: usize = Register::Ch1Overflow as usize;
const RX_BBF_C3_MSB: usize = Register::RxBbfC3Msb as usize;
const RX_BBF_C3_LSB: usize = Register::RxBbfC3Lsb as usize;
const RX_BBF_R2346: usize = Register::RxBbfR2346 as usize;
const RX_CAL_STATUS: usize = Register::RxCalStatus as usize;
const TX_CAL_STATUS: usize = Register::TxCalStatus as usize;
const RX_VCO_LOCK: usize = Register::RxCpOverrangeVcoLock as usize;
const TX_VCO_LOCK: usize = Register::TxCpOverrangeVcoLock as usize;
const QUAD_CAL_STATUS_TX1: usize = Register::QuadCalStatusTx1 as usize;
const QUAD_CAL_STATUS_TX2: usize = Register::QuadCalStatusTx2 as usize;

/// Register file behind the SPI traits of `embedded-hal` and `embedded-hal-async`.
pub struct DeviceMock {
    regs: [u8; REG_COUNT],
    frame: Frame,
}

/// Progress of the current chip select frame.
enum Frame {
    Header { first_byte: Option<u8> },
    Data { write: bool, addr: usize },
}

impl DeviceMock {
    pub fn new() -> Self {
        let mut regs = [0; REG_COUNT];
        regs[PRODUCT_ID] = PRODUCT_ID_AD9361_REV2;
        // Component values the RX BB filter tune leaves behind. The RX ADC setup divides by a
        // value derived from them, so they must not be zero.
        regs[RX_BBF_C3_MSB] = 0;
        regs[RX_BBF_C3_LSB] = 0x10;
        regs[RX_BBF_R2346] = 4;
        Self {
            regs,
            frame: Frame::Header { first_byte: None },
        }
    }

    fn read_reg(&self, addr: usize) -> u8 {
        match addr {
            BB_PLL_LOCK => self.regs[addr] | 1 << 7,
            // CP calibration valid and done.
            RX_CAL_STATUS | TX_CAL_STATUS => self.regs[addr] | 1 << 7 | 1 << 5,
            RX_VCO_LOCK | TX_VCO_LOCK => self.regs[addr] | 1 << 1,
            // Both the LO and the SSB calibration converged.
            QUAD_CAL_STATUS_TX1 | QUAD_CAL_STATUS_TX2 => 0b11,
            STATE => self.ensm_state(),
            _ => self.regs[addr],
        }
    }

    fn write_reg(&mut self, addr: usize, value: u8) {
        self.regs[addr] = match addr {
            // Calibrations finish instantly, so their start bits clear right away.
            CALIBRATION_CTRL => 0,
            _ => value,
        };
    }

    /// Derives the ENSM state from the force bits of the first ENSM config register.
    fn ensm_state(&self) -> u8 {
        let config = self.regs[ENSM_CONFIG_1];
        let force_rx_on = config & (1 << 6) != 0;
        let force_tx_on = config & (1 << 5) != 0;
        let to_alert = config & 1 != 0;
        match (force_rx_on, force_tx_on, to_alert) {
            (true, _, _) => 0x8,
            (_, true, _) => 0xA,
            (_, _, true) => 0x5,
            _ => 0x0,
        }
    }

    /// Clocks one byte through the device and returns the byte on MISO.
    fn clock_byte(&mut self, mosi: u8) -> u8 {
        match self.frame {
            Frame::Header { first_byte: None } => {
                self.frame = Frame::Header {
                    first_byte: Some(mosi),
                };
                0
            }
            Frame::Header {
                first_byte: Some(high),
            } => {
                let header = u16::from_be_bytes([high, mosi]);
                self.frame = Frame::Data {
                    write: header & AD_WRITE != 0,
                    addr: (header & 0x3FF) as usize,
                };
                0
            }
            Frame::Data { write, addr } => {
                // Multi-byte transfers walk the register addresses downwards.
                self.frame = Frame::Data {
                    write,
                    addr: addr.wrapping_sub(1) % REG_COUNT,
                };
                if write {
                    self.write_reg(addr, mosi);
                    0
                } else {
                    self.read_reg(addr)
                }
            }
        }
    }

    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) {
        self.frame = Frame::Header { first_byte: None };
        for operation in operations {
            match operation {
                Operation::Read(read) => read.iter_mut().for_each(|b| *b = self.clock_byte(0)),
                Operation::Write(write) => write.iter().for_each(|b| {
                    self.clock_byte(*b);
                }),
                Operation::Transfer(read, write) => {
                    for i in 0..read.len().max(write.len()) {
                        let miso = self.clock_byte(write.get(i).copied().unwrap_or(0));
                        if let Some(b) = read.get_mut(i) {
                            *b = miso;
                        }
                    }
                }
                Operation::TransferInPlace(buf) => buf.iter_mut().for_each(|b| {
                    *b = self.clock_byte(*b);
                }),
                Operation::DelayNs(_) => {}
            }
        }
    }
}

impl ErrorType for DeviceMock {
    type Error = Infallible;
}

impl embedded_hal::spi::SpiDevice for DeviceMock {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        DeviceMock::transaction(self, operations);
        Ok(())
    }
}

impl embedded_hal_async::spi::SpiDevice for DeviceMock {
    async fn transaction(
        &mut self,
        operations: &mut [Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        DeviceMock::transaction(self, operations);
        Ok(())
    }
}

pub struct ResetPin;

impl embedded_hal::digital::ErrorType for ResetPin {
    type Error = Infallible;
}

impl embedded_hal::digital::OutputPin for ResetPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// The mock has no timing, so all delays return immediately.
pub struct NoDelay;

impl embedded_hal::delay::DelayNs for NoDelay {
    fn delay_ns(&mut self, _ns: u32) {}
}

impl embedded_hal_async::delay::DelayNs for NoDelay {
    async fn delay_ns(&mut self, _ns: u32) {}
}
