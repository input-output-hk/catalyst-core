# Simple dockerfile example to build a catalyst voting backend 

FROM ubuntu:18.04
LABEL MAINTAINER IOHK
LABEL description="Catalyst Disruption backend"

ARG PREFIX=/app
ENV ENV_PREFIX=${PREFIX}

ARG VITUP_CONF_PATH_ARG=./example/mock/config.yaml
ENV VITUP_CONF_PATH=${VITUP_CONF_PATH_ARG}

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

RUN cd ${ENV_PREFIX}/catalyst-core/src/vit-testing/vitup && \
    ~/.cargo/bin/cargo install --locked --path . 

ENV PATH="/root/.cargo/bin/:${PATH}"

EXPOSE 80 8080

WORKDIR ${ENV_PREFIX}/catalyst-core/src/vit-testing/vitup
RUN mkdir -p ./mock

ENTRYPOINT ~/.cargo/bin/vitup start mock
