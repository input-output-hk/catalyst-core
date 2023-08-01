#! /bin/sh
set -eux

rustup toolchain list
cargo fmt -- --check
cargo clippy
