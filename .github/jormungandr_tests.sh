set -ex

RUSTFLAGS="-D warnings" 
RUST_BACKTRACE=1
CARGO_FLAGS="--verbose --locked"
CARGO_INCREMENTAL=0

uniffi-bindgen --version
which uniffi-bindgen
cargo tree -p uniffi_build
cargo tree -p uniffi
cargo build --all-targets --locked
cargo nextest run --no-fail-fast --partition hash:$1/$2 

