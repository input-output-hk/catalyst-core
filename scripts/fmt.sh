#! /bin/sh
set -eux

cargo fmt
cargo clippy --fix --allow-dirty

