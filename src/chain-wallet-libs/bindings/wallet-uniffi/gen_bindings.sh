#!/usr/bin/env sh

 uniffi-bindgen generate -l kotlin src/lib.udl --config-path uniffi.toml -o codegen/kotlin
