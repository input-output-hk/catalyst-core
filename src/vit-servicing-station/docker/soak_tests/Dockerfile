# Simple dockerfile example to build a vit server

FROM ubuntu:18.04
LABEL MAINTAINER IOHK
LABEL description="VIT server soak tests"

ARG PREFIX=/app
ENV ENV_PREFIX="vit_server_env"

# prepare the environment
RUN apt-get update && \
    apt-get install -y git curl && \
    mkdir -p ${ENV_PREFIX} && \
    cd ${ENV_PREFIX} && \
    git clone --recurse-submodules https://github.com/input-output-hk/vit-servicing-station src
    
#install rustup
RUN  apt-get install -y build-essential pkg-config libssl-dev && \
    bash -c "curl https://sh.rustup.rs -sSf | bash -s -- -y" && \
     ~/.cargo/bin/rustup install stable && \
    ~/.cargo/bin/rustup default stable


# install the node and jcli from source
RUN cd ${ENV_PREFIX}/src && \
    git submodule update --init --recursive && \
    ~/.cargo/bin/cargo build --all --release --locked

WORKDIR ${ENV_PREFIX}/src/vit-servicing-station-tests
RUN ["bash", "-c", "~/.cargo/bin/cargo test rest_load_long --features  non-functional --release -- --nocapture"]
