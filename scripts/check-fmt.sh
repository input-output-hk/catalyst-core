#! /bin/sh
set -eux

cargo fmt -- --check
cargo clippy
cargo clippy --all-features
