# Set the Earthly version to 0.7
VERSION 0.7
FROM debian:stable-slim

# Installs and configures the Rust toolchain
rust-toolchain:
    ARG user=user
    ARG uid=1000
    ARG gid=$uid

    # Install dependencies
    RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        sudo

    # Create a user
    RUN groupadd -g $gid $user && \
        useradd -u $uid -g $gid -G sudo -m $user -s /bin/bash

    # Setup sudo
    RUN sed -i 's/%sudo.*ALL/%sudo   ALL=(ALL:ALL) NOPASSWD:ALL/' /etc/sudoers

    WORKDIR /work
    ARG rustup_url="https://static.rust-lang.org/rustup/archive/1.26.0/x86_64-unknown-linux-gnu/rustup-init"
    ENV PATH="${HOME}/.cargo/bin:${PATH}"

    # Install build dependencies
    RUN sudo apt-get update && \
        sudo apt-get install -y --no-install-recommends \
        build-essential \
        libssl-dev \
        libpq-dev \
        libsqlite3-dev \
        protobuf-compiler

    # Download and verify the Rustup installer
    RUN curl \
        --fail \
        --remote-name \
        --location \
        $rustup_url
    RUN curl \
        --fail \
        --remote-name \
        --location \
        $rustup_url.sha256
    RUN sed -i 's| .*rustup-init| rustup-init|' rustup-init.sha256 && \
        sha256sum --check rustup-init.sha256

    # Install the Rust toolchain
    RUN chmod +x rustup-init && \
        ./rustup-init -y --default-toolchain none

    # Cleanup
    RUN rm rustup-init rustup-init.sha256

    # Force rustup to initialize the toolchain from the rust-toolchain file
    COPY rust-toolchain .
    RUN rustup show

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
    RUN cargo chef cook --release
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home
    SAVE IMAGE --cache-hint

# This is the default builder that all other builders should inherit from
builder:
    FROM +rust-toolchain
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
    ARG tag=latest

    # Determine the final registry to push to
    IF [ "$registry" = "" ]
        ARG registry_final=$registry
    ELSE
        ARG registry_final=${registry}/
    END

    # Build and tag all Docker images
    BUILD ./containers/event-db-migrations+docker --tag=$tag --registry=$registry_final
    BUILD ./containers/event-db-graphql+docker --tag=$tag --registry=$registry_final

    # Build crate images from the workspace
    BUILD ./src/jormungandr/jormungandr+docker --tag=$tag --registry=$registry_final
    BUILD ./src/jormungandr/jcli+docker --tag=$tag --registry=$registry_final
    BUILD ./src/catalyst-toolbox/catalyst-toolbox+docker --tag=$tag --registry=$registry_final
    BUILD ./src/voting-tools-rs+docker --tag=$tag --registry=$registry_final
    BUILD ./src/cat-data-service+docker --tag=$tag --registry=$registry_final

    BUILD ./services/voting-node+docker --tag=$tag --registry=$registry_final
    BUILD ./utilities/ideascale-importer+docker --tag=$tag --registry=$registry_final

all-with-tags:
    FROM +tag-workspace

    ARG registry

    ARG VERSION=$(svu --pattern="v[0-9]*.[0-9]*" current)
    ARG TIMESTAMP=$(TZ=UTC date +"%Y%m%d%H%M%S")

    ARG TAG_VER=${VERSION}-${TIMESTAMP}
    ARG TAG_HASH=$(git rev-parse HEAD)

    BUILD +all --tag=${TAG_VER} --registry=${registry}
    BUILD +all --tag=${TAG_HASH} --registry=${registry}

# Define the ci stage, which only builds the event-db-migrations Docker image for testing
ci:
    BUILD ./containers/event-db-migrations+test

# Define the test stage, which runs the Rust project's tests
test:
    FROM +devshell
    RUN cargo --version

tag-workspace:
    ARG SVU_VERSION=1.10.2
    WORKDIR /work

    RUN apt-get update && apt-get install -y curl git
    RUN curl \
        --fail \
        --remote-name \
        --location \
        "https://github.com/caarlos0/svu/releases/download/v${SVU_VERSION}/svu_${SVU_VERSION}_linux_amd64.deb"
    RUN dpkg -i svu_${SVU_VERSION}_linux_amd64.deb

    COPY .git .
    RUN git tag -l
    SAVE IMAGE --cache-hint