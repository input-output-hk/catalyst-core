# Simple dockerfile example to build a jormungandr voting backend 

FROM ubuntu:18.04
LABEL MAINTAINER IOHK
LABEL description="Jormungandr Voting backend"

ARG PREFIX=/app
ENV ENV_PREFIX=${PREFIX}

# prepare the environment
RUN apt-get update && \
    apt-get install -y git curl && \
    mkdir -p ${ENV_PREFIX}

#install rustup
RUN  apt-get install -y build-essential pkg-config libssl-dev && \
    bash -c "curl https://sh.rustup.rs -sSf | bash -s -- -y" && \
     ~/.cargo/bin/rustup install stable && \
    ~/.cargo/bin/rustup default stable

RUN cd ${ENV_PREFIX} && \
    git clone https://github.com/input-output-hk/catalyst-core && \
    cd catalyst-core && \
	git checkout main

# install the vit-servicing-station-server from source
RUN cd ${ENV_PREFIX}/catalyst-core/src/vit-servicing-station && \
    ~/.cargo/bin/cargo install --locked --force --path vit-servicing-station-server
    
# install the node and jcli from source
RUN cd ${ENV_PREFIX}/catalyst-core/src/jormungandr && \
    ~/.cargo/bin/cargo install --locked --path jormungandr && \
    ~/.cargo/bin/cargo install --locked --path jcli

# install the node and jcli from source
RUN cd ${ENV_PREFIX}/catalyst-core/src/vit-testing && \
    ~/.cargo/bin/cargo install --locked --path vitup && \
    ~/.cargo/bin/cargo install --locked --path valgrind

ENV PATH="/root/.cargo/bin/:${PATH}"

EXPOSE 80 8000 8001 8002 8003 8004 8005 8006 8007 8008 8009 8010 3030

WORKDIR ${ENV_PREFIX}/catalyst-core/src/vit-testing/vitup
RUN mkdir -p mock
ENTRYPOINT ~/.cargo/bin/vitup start quick --jormungandr ~/.cargo/bin/jormungandr --jcli ~/.cargo/bin/jcli --endpoint 0.0.0.0:8080 --mode service --https
