# Simple dockerfile example to build a catalyst voting backend 

FROM ubuntu:18.04@sha256:1e32b9c52e8f22769df41e8f61066c77b2b35b0a423c4161c0e48eca2fd24f75
LABEL MAINTAINER IOHK
LABEL description="Catalyst Snapshot Service"

ARG PREFIX=/app
ENV ENV_PREFIX=${PREFIX}

# prepare the environment
RUN apt-get update && \
    apt-get install -y git curl && \
    mkdir -p ${ENV_PREFIX}

#install rustup
RUN  apt-get install -y build-essential pkg-config libssl-dev libpq-dev && \
    bash -c "curl https://sh.rustup.rs -sSf | bash -s -- -y" && \
     ~/.cargo/bin/rustup install stable && \
    ~/.cargo/bin/rustup default stable
	
RUN cd ${ENV_PREFIX} && \
    git clone https://github.com/input-output-hk/catalyst-core && \
    cd catalyst-core && \
	git checkout main

RUN cd ${ENV_PREFIX}/catalyst-core/src/vit-testing/snapshot-trigger-service && \
    ~/.cargo/bin/cargo install --locked --path . --bin snapshot-trigger-service

RUN cd ${ENV_PREFIX}/catalyst-core/src/voting-tools-rs && \
    ~/.cargo/bin/cargo install --locked --path . --bin snapshot_tool

ENV PATH="/root/.cargo/bin/:${PATH}"

EXPOSE 8080

WORKDIR ${ENV_PREFIX}/catalyst-core/src/vit-testing/snapshot-trigger-service

COPY entrypoint.sh /
ENTRYPOINT ["/entrypoint.sh"]
CMD ~/.cargo/bin/snapshot-trigger-service --config /config.yaml
