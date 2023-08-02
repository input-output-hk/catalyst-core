FROM rust:1.71.0-slim-bullseye
WORKDIR /usr/src/catalyst-core
COPY . .
RUN apt-get update && \
    apt-get install -y build-essential pkg-config protobuf-compiler \
    libssl-dev libpq-dev libsqlite3-dev
RUN cargo check --locked --release -p jormungandr -p jcli -p vit-servicing-station-server
