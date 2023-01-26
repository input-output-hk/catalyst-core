# Simple dockerfile example to build a vit server

FROM ubuntu:18.04
LABEL MAINTAINER IOHK
LABEL description="Vit servicing station server"

ARG PREFIX=/app
ENV ENV_PREFIX="vit_server_env"

COPY database.db /data/database.db
COPY block0.bin /data/block0.bin


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
    ~/.cargo/bin/cargo build --all --release --locked && \
	~/.cargo/bin/cargo install --path vit-servicing-station-server
	

CMD ["bash", "-c", "~/.cargo/bin/vit-servicing-station-server --db-url /data/database.db --block0-path /data/block0.bin --log-output-path vit-servicing-station.log --log-level info"]
