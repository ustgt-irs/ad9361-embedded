//! Brings up the driver against an in-memory AD9361 model, once with the blocking and once with
//! the async API.
//!
//! Run with `cargo run --example bringup`.
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

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

fn run_blocking(config: &ad9361_embedded::config::ConfigValidated) {
    let mut ad9361 = ad9361_embedded::Ad9361Uninit::new(DeviceMock::new(), ResetPin, NoDelay);
    ad9361.reset().unwrap();
    println!(
        "blocking: product ID {:?}",
        ad9361.read_product_id().unwrap()
    );
    let (_driver, _clocks) = ad9361.init(config).expect("init failed");
    println!("blocking: init complete");
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
