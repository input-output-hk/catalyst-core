# Simple dockerfile example to build a catalyst voting backend 

FROM ubuntu:18.04
LABEL MAINTAINER IOHK
LABEL description="Catalyst Disruption backend"

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
	git checkout snapshot_create_mock

RUN cd ${ENV_PREFIX}/catalyst-core/src/vit-testing/vitup && \
    ~/.cargo/bin/cargo install --locked --path . 

ENV PATH="/root/.cargo/bin/:${PATH}"

EXPOSE 80 8000 8001 8002 8003 8004 8005 8006 8007 8008 8009 8010 3030

WORKDIR ${ENV_PREFIX}/catalyst-core/src/vit-testing/vitup
RUN mkdir -p ./mock

ENTRYPOINT ~/.cargo/bin/vitup start mock --config ./example/mock/https.yaml
