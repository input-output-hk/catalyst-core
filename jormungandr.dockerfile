# Simple dockerfile example to build a jormungandr and jcli

FROM ubuntu:latest
LABEL MAINTAINER IOHK
LABEL description="Jormungandr latest"

ARG APP_PATH=/app
ARG REST_PORT=8448
ENV ENV_BUILD=false
ARG VER=v0.2.3
ENV ENV_VER=${VER}
ENV RUST_VERSION=1.65.0
ENV WORKSPACE=/workspace

COPY . /workspace
WORKDIR ${WORKSPACE}

# prepare the environment
RUN echo "[INFO] - Preparing the environment" && \
    apt-get update && \
    apt-get install -y curl git build-essential pkg-config \
                       protobuf-compiler libssl-dev libpq-dev libsqlite3-dev

#install rustup
RUN echo "[INFO] - Installing Rust" && \
    bash -c "curl https://sh.rustup.rs -sSf | bash -s -- -y" && \
    export PATH=$HOME/.cargo/bin:$PATH && \
    rustup install ${RUST_VERSION} && \
    rustup default ${RUST_VERSION} && \
    cargo install --locked --path src/jormungandr/jormungandr && \
    cargo install --locked --path src/jormungandr/jcli && \
    mkdir -p ${APP_PATH}/bin && \
    cp $HOME/.cargo/bin/jormungandr ${APP_PATH}/bin && \
    cp $HOME/.cargo/bin/jcli ${APP_PATH}/bin

# cleanup
RUN apt-get remove --purge --auto-remove -y git curl build-essential pkg-config && \
    apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf $HOME/.rustup

WORKDIR ${APP_PATH}
# TODO: Add files to this path needed for the node
EXPOSE ${REST_PORT}

CMD [ "./bin/jormungandr", "--help" ]
