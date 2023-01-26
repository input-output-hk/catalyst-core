# Simple dockerfile example to build a vit server

FROM ubuntu:latest
LABEL MAINTAINER IOHK
LABEL description="Vit servicing station server"

ARG APP_PATH=/app
ENV RUST_VERSION=1.65.0
ENV WORKSPACE=/workspace

COPY . /workspace
WORKDIR ${WORKSPACE}

# prepare the environment
RUN echo "[INFO] - Preparing the environment" && \
    apt-get update && \
    apt-get install -y curl git build-essential pkg-config \
                       protobuf-compiler libssl-dev libpq-dev libsqlite3-dev
    
#install rust and crate
RUN bash -c "curl https://sh.rustup.rs -sSf | bash -s -- -y" && \
    export PATH=$HOME/.cargo/bin:$PATH && \
    rustup install ${RUST_VERSION} && \
    rustup default ${RUST_VERSION} && \
    cargo install --locked --path src/vit-servicing-station/vit-servicing-station-server && \
    mkdir -p ${APP_PATH}/bin && \
    cp $HOME/.cargo/bin/vit-servicing-station-server ${APP_PATH}/bin

# cleanup
RUN apt-get remove --purge --auto-remove -y git curl build-essential pkg-config && \
    apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf $HOME/.rustup

WORKDIR ${APP_PATH}
# TODO: Add files to this path needed for the node

CMD [ "./bin/vit-servicing-station-server", "--help" ]
