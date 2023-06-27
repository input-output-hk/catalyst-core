# Set the Earthly version to 0.7
VERSION 0.7
FROM debian:stable-slim

rust-toolchain:
    FROM rust:1.65-slim-bullseye

# Installs Cargo chef
install-chef:
    FROM +rust-toolchain
    RUN cargo install --debug cargo-chef

# Prepares the local cache
prepare-cache:
    FROM +install-chef
    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo chef prepare
    SAVE ARTIFACT recipe.json
    SAVE IMAGE --cache-hint

# Builds the local cache
build-cache:
    FROM +install-chef
    COPY +prepare-cache/recipe.json ./

    # Install build dependencies
    RUN apt-get update && \
        apt-get install -y --no-install-recommends \
        build-essential \
        libssl-dev \
        libpq-dev \
        libsqlite3-dev \
        pkg-config \
        protobuf-compiler

    RUN cargo chef cook --release
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home
    SAVE IMAGE --cache-hint

# This is the default builder that all other builders should inherit from
builder:
    FROM +rust-toolchain
    # Install build dependencies
    RUN apt-get update && \
        apt-get install -y --no-install-recommends \
        build-essential \
        libssl-dev \
        libpq-dev \
        libsqlite3-dev \
        pkg-config \
        protobuf-compiler
    RUN rustup component add rustfmt
    COPY --dir src Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    SAVE ARTIFACT src

# This is the default deployment that all other deployments should inherit from
deployment:
    FROM debian:stable-slim

# Define the all stage, which builds and tags all Docker images
all:
    ARG EARTHLY_CI
    ARG EARTHLY_GIT_SHORT_HASH
    ARG registry
    ARG tag="catalyst-fund9.1"

    # Determine the final registry to push to
    IF [ "$registry" = "" ]
        ARG registry_final=$registry
    ELSE
        ARG registry_final=${registry}/
    END

    # Build crate images from the workspace
    BUILD ./src/jormungandr/jormungandr+docker --tag=$tag --registry=$registry_final
    BUILD ./src/jormungandr/jcli+docker --tag=$tag --registry=$registry_final
    BUILD ./src/jormungandr/explorer+docker --tag=$tag --registry=$registry_final
