# Simple dockerfile example to build a jormungandr voting backend 

FROM ubuntu:18.04
LABEL MAINTAINER IOHK
LABEL description="Jormungandr Voting backend"

ARG PREFIX=/app
ENV ENV_PREFIX=${PREFIX}

ARG DATA=${PREFIX}/data
ENV ENV_DATA=${DATA}

ARG SLOT_DURATION=1
ENV ENV_SLOT_DURATION=${SLOT_DURATION}

ARG SLOTS_IN_EPOCH=60
ENV ENV_SLOTS_IN_EPOCH=${SLOTS_IN_EPOCH}

ARG VOTE_START_EPOCH=1 
ENV ENV_VOTE_START_EPOCH=${VOTE_START_EPOCH}

ARG TALLY_START_EPOCH=5
ENV ENV_TALLY_START_EPOCH=${TALLY_START_EPOCH}

ARG TALLY_END_EPOCH=10
ENV ENV_TALLY_END_EPOCH=${TALLY_END_EPOCH}

#To Enable private voting change line to 
#ARG PRIVATE="--private"
ARG PRIVATE=""
ENV ENV_PRIVATE=${PRIVATE}

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
    git clone https://github.com/input-output-hk/jormungandr && \
	cd jormungandr && \
	git checkout cbd280b4652bd397c34b25caff28d816c6be3941

RUN cd ${ENV_PREFIX} && \
    git clone https://github.com/input-output-hk/vit-servicing-station && \
	cd vit-servicing-station && \
	git checkout 75a3c30211892c32a0edf1ff6b4b368d5e3a0e82
	
 RUN cd ${ENV_PREFIX} && \
    git clone https://github.com/input-output-hk/vit-testing && \
    cd vit-testing && \
	git checkout ffb17a3864ca97d9e8b73bd74a3700c9cba9e431
    
# install the node and jcli from source
RUN cd ${ENV_PREFIX}/vit-servicing-station && \
    ~/.cargo/bin/cargo install --locked --force --path vit-servicing-station-server
    
# install the node and jcli from source
RUN cd ${ENV_PREFIX}/jormungandr && \
    ~/.cargo/bin/cargo install --locked --path jormungandr && \
    ~/.cargo/bin/cargo install --locked --path jcli

# install the node and jcli from source
RUN cd ${ENV_PREFIX}/vit-testing && \
    ~/.cargo/bin/cargo install --locked --path vitup && \
    ~/.cargo/bin/cargo install --locked --path iapyx

ENV PATH="/root/.cargo/bin/:${PATH}"

EXPOSE 80 8000 8001 8002 8003 8004 8005 8006 8007 8008 8009 8010 3030

WORKDIR ${ENV_PREFIX}/vit-testing/vitup
RUN mkdir -p mock
ENTRYPOINT ~/.cargo/bin/vitup start quick --jormungandr ~/.cargo/bin/jormungandr --jcli ~/.cargo/bin/jcli --endpoint 0.0.0.0:80 --vote-start-epoch $ENV_VOTE_START_EPOCH --tally-start-epoch $ENV_TALLY_START_EPOCH --tally-end-epoch $ENV_TALLY_END_EPOCH --slot-duration $ENV_SLOT_DURATION --slots-in-epoch $ENV_SLOTS_IN_EPOCH --progress-bar-mode standard --root-dir $ENV_DATA $ENV_PRIVATE --mode service --version "2.0"
