# Run every check CI runs.
all: check test msrv cross-check check-fmt clippy docs

check:
    cargo check --release
    cargo check --release --all-features

test:
    cargo test
    cargo test --all-features

# The MSRV is specified in Cargo.toml.
msrv:
    cargo +1.88 check --release
    cargo +1.88 check --release --all-features

cross-check:
    cargo check --release --target thumbv7em-none-eabihf --no-default-features
    cargo check --release --target thumbv7em-none-eabihf --all-features
    cargo check --release --target armv7a-none-eabihf --no-default-features
    cargo check --release --target armv7a-none-eabihf --all-features

clippy:
    cargo clippy --all-features -- -D warnings

fmt:
    cargo fmt --all

check-fmt:
    cargo fmt --all -- --check

docs:
    RUSTDOCFLAGS="--cfg docsrs -D warnings -Z unstable-options --generate-link-to-definition" cargo +nightly doc --all-features --no-deps

docs-html:
    RUSTDOCFLAGS="--cfg docsrs -Z unstable-options --generate-link-to-definition" cargo +nightly doc --all-features --no-deps --open
