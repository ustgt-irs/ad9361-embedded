//! Brings up the driver against an in-memory AD9361 model, once with the blocking and once with
//! the async API.
//!
//! Run with `cargo run --example bringup`.
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

use ad9361_embedded::clocks::{ClockConfig, Clocks};
use mock::{DeviceMock, NoDelay, ResetPin};

mod config;
mod mock;

/// All futures of the mock are ready immediately, so polling in a loop is enough.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
    }
}

fn print_clocks(config: &ClockConfig, clocks: &Clocks) {
    println!("clocks:");
    println!("  external reference: {} Hz", clocks.ext_clk());
    println!("  BB reference:       {} Hz", clocks.bb_refclk());
    println!(
        "  BB PLL:             {} Hz (N={}, F={:?}, Icp={} uA)",
        clocks.bb_pll(),
        config.bb_pll.integer_freq_word,
        config.bb_pll.fractional_freq_word,
        config.bb_pll.charge_pump.current()
    );
    println!(
        "  ADC:                {} Hz (divisor {:?})",
        clocks.adc(),
        config.adc
    );

    let rx = clocks.rx();
    println!(
        "  RX synth reference: {} Hz, VCO {:?} Hz, LO {} Hz",
        rx.synth_ref(),
        rx.pll_vco(),
        rx.lo()
    );
    println!(
        "  RX path:            ADC {} Hz -> HB3 {} Hz -> HB2 {} Hz -> HB1 {} Hz -> sample {} Hz",
        clocks.adc(),
        rx.hb3(),
        rx.hb2(),
        rx.hb1(),
        rx.rx_sample()
    );

    let tx = clocks.tx();
    println!(
        "  TX synth reference: {} Hz, VCO {:?} Hz, LO {} Hz",
        tx.synth_ref(),
        tx.pll_vco(),
        tx.lo()
    );
    println!(
        "  TX path:            sample {} Hz -> FIR {} Hz -> HB1 {} Hz -> HB2 {} Hz -> DAC {} Hz",
        tx.tx_sample(),
        tx.tx_fir(),
        tx.hb1(),
        tx.hb2(),
        tx.dac_clk()
    );
}

fn run_blocking(config: &ad9361_embedded::config::ConfigValidated) {
    let mut ad9361 = ad9361_embedded::Ad9361Uninit::new(DeviceMock::new(), ResetPin, NoDelay);
    ad9361.reset().unwrap();
    println!(
        "blocking: product ID {:?}",
        ad9361.read_product_id().unwrap()
    );
    let (_driver, clocks) = ad9361.init(config).expect("init failed");
    println!("blocking: init complete");
    print_clocks(&config.0.clock, &clocks);
}

fn run_async(config: &ad9361_embedded::config::ConfigValidated) {
    block_on(async {
        let mut ad9361 =
            ad9361_embedded::asynch::Ad9361Uninit::new(DeviceMock::new(), ResetPin, NoDelay);
        ad9361.reset().await.unwrap();
        println!(
            "async: product ID {:?}",
            ad9361.read_product_id().await.unwrap()
        );
        let (_driver, _clocks) = ad9361.init(config).await.expect("init failed");
        println!("async: init complete");
    });
}

fn main() {
    let config = config::build();
    run_blocking(&config);
    run_async(&config);
}
