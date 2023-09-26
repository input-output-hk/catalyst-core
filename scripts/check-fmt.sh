#! /bin/sh
set -eux


cargo fmt -- --check
cargo clippy
