[![Crates.io](https://img.shields.io/crates/v/ad9361-embedded)](https://crates.io/crates/ad9361-embedded)
[![docs.rs](https://img.shields.io/docsrs/ad9361-embedded)](https://docs.rs/ad9361-embedded)
[![ci](https://github.com/ustgt-irs/ad9361-embedded/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/ustgt-irs/ad9361-embedded/actions/workflows/ci.yml)

Pure-Rust AD9361 driver
======================

`ad9361-embedded` is a `no_std` Rust driver for the
[Analog Devices AD9361](https://www.analog.com/en/products/ad9361.html) RF agile transceiver.

The driver only depends on abstractions:

- The SPI bus, the reset pin and all timing are provided through the
  [`embedded-hal`](https://crates.io/crates/embedded-hal) traits `SpiDevice`, `OutputPin` and
  `DelayNs`.
- The same source code provides a **blocking** API based on `embedded-hal` and an **async** API
  based on [`embedded-hal-async`](https://crates.io/crates/embedded-hal-async). Both are generated
  from a single implementation using [`bisync2`](https://crates.io/crates/bisync2). They live in
  the `blocking` and `asynch` modules. The crate root re-exports the blocking API.
- The only source of time is `DelayNs`. The driver needs no timer, no executor and no operating
  system, and it does not depend on `std`. Calibration timeouts are implemented by polling with
  delays. The `asynch` module expects an async [`DelayNs`](https://docs.rs/embedded-hal-async/latest/embedded_hal_async/delay/trait.DelayNs.html)
  while the blocking module expects a [blocking one](https://docs.rs/embedded-hal/latest/embedded_hal/delay/trait.DelayNs.html).

The crate does not contain any `unsafe` code. The register map is described with typed bitfields
in the `regs` module.

# Features

`ad9361-embedded` currently supports the following features:

- Device bring-up: reset, clock chain (BB PLL, ADC and DAC clocks, half-band and FIR stages), RX
  and TX RF PLLs, and the calibrations (BB and RF DC offset, BB analog filters, TIA, RX ADC and TX
  quadrature).
- Calculation and validation of the clock configuration for a target sample rate, and updating
  the clock configuration and sample rate at run-time with `Ad9361::update_rf_clocks`.
- Gain control: manual, slow attack, fast attack and hybrid AGC together with the RX gain tables.
- Loading of the TX and RX FIR filter coefficients.
- Digital interface configuration for LVDS and CMOS.
- RSSI, AUXADC, AUXDAC, GPO, control output pins, external LNA control and the TX monitor.
- Enable state machine (ENSM) control and TX muting.

The following features have not been implemented yet. PRs or notifications for demand are welcome!

- Fast lock profiles
- Changing the LO frequencies at run-time

## Default features

- `axi-tune`: Enables the tuning of the digital interface delays (`Ad9361::digital_tune`). This
  requires access to the AXI ADC and DAC of the Analog Devices HDL core, which is provided by the
  [`axi-ad9361`](https://crates.io/crates/axi-ad9361) crate.

## Optional features

- [`defmt`](https://defmt.ferrous-systems.com/): Adds the
  [`defmt::Format`](https://defmt.ferrous-systems.com/format) derive on the register and
  configuration types.

# Tests

Run the tests with all features enabled:

```sh
cargo test --all-features
```

# Coverage

Coverage can be generated using [`llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov). If you
have not done so already, install the tool:

```sh
cargo +stable install cargo-llvm-cov --locked
```

After this, you can run `cargo llvm-cov nextest` to run all the tests and display coverage.

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Parts of this crate are derived from the AD9361 driver of the Analog Devices no-OS library, which
is licensed under the 3-clause BSD license. The copyright notice and the license terms of these
parts are kept in [LICENSE-ADI](LICENSE-ADI) and [NOTICE](NOTICE).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
