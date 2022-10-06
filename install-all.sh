#!/usr/bin/env bash

# Catalyst Toolbox Build/Test/Install
catalyst_toolbox() {
    pushd src/catalyst-toolbox
    cargo build
    cargo test
    cargo install --locked --path catalyst-toolbox
    popd
}

# chain-libs Build/Test/nothing to install
chain_libs() {
    pushd src/chain-libs
    cargo build
    cargo test
    popd
}

# Just build the JS bindings for chain-wallet-libs
chain_wallet_libs() {
    pushd src/chain-wallet-libs/bindings/wallet-js
    wasm-pack build -d pkg
    jsdoc pkg -c ../../jsdoc.json -d pkg/doc -R README.md
    popd
}

# Jormungandr Build/Test/Install
jormungandr() {
    pushd src/jormungandr
    cargo build
    cargo test
    cargo install --locked --path jormungandr # --features systemd # (on linux with systemd)
    cargo install --locked --path jcli
    popd
}

# jortestkit Build/Test/nothing to install
jortestkit() {
    pushd src/jortestkit
    cargo build
    cargo test
    popd
}

# vit-servicing-station Build/Test/Install
vit_servicing_station() {
    pushd src/vit-servicing-station
    cargo build
    cargo test
    cargo install --locked --path vit-servicing-station-cli
    cargo install --locked --path vit-servicing-station-server
    popd
}

# vit-testing Build/Test/Install
vit_testing() {
    pushd src/vit-testing
    cargo build
    cargo test
    cargo install --locked --path iapyx
    cargo install --locked --path vitup
    cargo install --locked --path valgrind
    cargo install --locked --path mainnet-tools
    cargo install --locked --path registration-service
    cargo install --locked --path registration-verify-service
    cargo install --locked --path snapshot-trigger-service
    popd
}


# voting-tools-rs Build/Test/Install
voting_tools_rs() {
    pushd src/voting-tools-rs
    cargo build
    cargo test --no-default-features
    cargo install --locked --path .
    popd
}


catalyst_toolbox
chain_libs
chain_wallet_libs
jormungandr
jortestkit
vit_servicing_station
vit_testing
voting_tools_rs
