VERSION 0.7

build-rust:
    FROM rust:1.65
    WORKDIR /catalyst-core
    RUN rustup component add rustfmt

install-chef:
    FROM +build-rust
    RUN cargo install --debug cargo-chef

prepare-cache:
    FROM +install-chef
    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo chef prepare
    SAVE ARTIFACT recipe.json

# Using cutoff-optimization to ensure cache hit (see examples/cutoff-optimization)
build-cache:
    FROM +install-chef
    RUN apt-get update && \
        apt-get install -y protobuf-compiler libssl-dev libpq-dev libsqlite3-dev
    COPY +prepare-cache/recipe.json ./
    RUN cargo chef cook --release
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home

jormungandr:
    FROM +build-rust
    COPY --dir src Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    RUN cargo build --locked --release --bin jormungandr
    SAVE ARTIFACT target/release/jormungandr jormungandr

jcli:
    FROM +build-rust
    COPY --dir src Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    RUN cargo build --locked --release --bin jcli
    SAVE ARTIFACT target/release/jcli jcli

catalyst-toolbox:
    FROM +build-rust
    COPY --dir src Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    RUN cargo build --locked --release --bin catalyst-toolbox
    SAVE ARTIFACT target/release/catalyst-toolbox catalyst-toolbox

cat-data-service:
    FROM +build-rust
    COPY --dir src Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    RUN cargo build --locked --release --bin cat-data-service
    SAVE ARTIFACT target/release/cat-data-service cat-data-service
