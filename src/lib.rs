#![no_std]

pub mod clocks;
pub mod config;
pub mod ctrl_output;
pub mod limits;
pub mod lut;
pub mod regs;
pub mod spi;
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
