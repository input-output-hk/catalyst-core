#!/usr/bin/env bash

# Loops through a list of crate names, using Cargo to build and test them
build_and_test() {
    for n in $1
    do
        cargo build -p $n
        cargo test -p $n
    done
}

# Catalyst Toolbox Build/Test/Install
catalyst_toolbox() {
    build_and_test "catalyst-toolbox catalyst-toolbox snapshot-lib"
    cargo install --locked --path src/catalyst-toolbox/catalyst-toolbox
}

# chain-libs Build/Test/nothing to install
chain_libs() {
    build_and_test "cardano-legacy-address cbor chain-addr chain-core chain-crypto chain-evm chain-impl-mockchain chain-network chain-ser chain-storage chain-time chain-vote imhamt imhamt memdump p256k1 ristretto shvzk sparse-array storage sumed25519 tally typed-bytes vrf"
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
    build_and_test "blockchain explorer hersir jcli jcli jcli_lib jormungandr jormungandr-automation jormungandr-integration-tests jormungandr-lib loki mjolnir rest_v0 settings thor"
    cargo install --locked --path src/jormungandr/jormungandr # --features systemd # (on linux with systemd)
    cargo install --locked --path src/jormungandr/jcli
}

# jortestkit Build/Test/nothing to install
jortestkit() {
    cargo build -p jortestkit
    cargo test -p jortestkit
}

# vit-servicing-station Build/Test/Install
vit_servicing_station() {
    build_and_test "vit-servicing-station-cli vit-servicing-station-lib vit-servicing-station-server vit-servicing-station-tests"
    cargo install --locked --path src/vit-servicing-station/vit-servicing-station-cli
    cargo install --locked --path src/vit-servicing-station/vit-servicing-station-server
}

# vit-testing Build/Test/Install
vit_testing() {
    build_and_test "iapyx integration-tests mainnet-tools registration-service registration-verify-service scheduler-service-lib signals-handler snapshot-trigger-service valgrind vitup"
    cargo install --locked --path src/vit-testing/iapyx
    cargo install --locked --path src/vit-testing/vitup
    cargo install --locked --path src/vit-testing/valgrind
    cargo install --locked --path src/vit-testing/mainnet-tools
    cargo install --locked --path src/vit-testing/registration-service
    cargo install --locked --path src/vit-testing/registration-verify-service
    cargo install --locked --path src/vit-testing/snapshot-trigger-service
}


# voting-tools-rs Build/Test/Install
voting_tools_rs() {
    cargo build -p voting_tools_rs
    cargo test --no-default-features -p voting_tools_rs
    cargo install --locked --path src/catalyst-toolbox/catalyst-toolbox
}


catalyst_toolbox
chain_libs
chain_wallet_libs
jormungandr
jortestkit
vit_servicing_station
vit_testing
voting_tools_rs
