set -ex

RUSTFLAGS="-D warnings" 
RUST_BACKTRACE=1
CARGO_FLAGS="--verbose --locked"
CARGO_INCREMENTAL=0

curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
cargo build --all-targets
cargo nextest run --no-fail-fast --partition hash:$1/$2 

