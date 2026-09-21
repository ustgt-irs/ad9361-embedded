//! Platform agnostic driver for the AD9361 RF transceiver with a blocking and an async API.
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

/// Clock tree calculation and configuration.
pub mod clocks;
/// Driver configuration and its validation.
pub mod config;
/// Reading of the control output pins.
// The per-signal accessors and type aliases follow the control output table of the datasheet.
#[allow(missing_docs)]
pub mod ctrl_output;
/// Hardware limits of the AD9361.
pub mod limits;
/// RF synthesizer lookup tables.
// The tables are ported from the no-OS driver.
#[allow(missing_docs)]
pub mod lut;
/// Register map with typed bitfields.
// Names follow the datasheet, so they are not documented one by one.
#[allow(missing_docs)]
pub mod regs;
/// SPI framing of the AD9361.
pub mod spi;
/// Types shared by the driver API.
pub mod types;

pub use spi::MODE as SPI_MODE;
pub use types::*;

/// Blocking driver, built on [embedded-hal]'s synchronous [`SpiDevice`]/[`DelayNs`] traits.
///
/// [embedded-hal]: https://docs.rs/embedded-hal
#[path = "."]
pub mod blocking {
    use bisync2::synchronous::*;
    pub use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

    // `blocking::inner` and `asynch::inner` are deliberately the same file, compiled
    // twice under different `bisync2` module scopes.
    #[allow(clippy::duplicate_mod)]
    mod inner;
    pub use inner::*;
}
pub use blocking::*;

/// Async driver, built on [embedded-hal-async]'s [`SpiDevice`]/[`DelayNs`] traits.
///
/// [embedded-hal-async]: https://docs.rs/embedded-hal-async
#[path = "."]
pub mod asynch {
    use bisync2::asynchronous::*;
    pub use embedded_hal::digital::OutputPin;
    pub use embedded_hal_async::{delay::DelayNs, spi::SpiDevice};

    // `blocking::inner` and `asynch::inner` are deliberately the same file, compiled
    // twice under different `bisync2` module scopes.
    #[allow(clippy::duplicate_mod)]
    mod inner;
    pub use inner::*;
}
