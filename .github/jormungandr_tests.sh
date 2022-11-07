set -ex

RUSTFLAGS="-D warnings" 
RUST_BACKTRACE=1
CARGO_FLAGS="--verbose --locked"
CARGO_INCREMENTAL=0

cat Cargo.lock
uniffi-bindgen --version
which uniffi-bindgen
cargo tree -i uniffi_build
cargo tree -i uniffi
cargo build --all-targets --locked
cargo nextest run --no-fail-fast --partition hash:$1/$2 

