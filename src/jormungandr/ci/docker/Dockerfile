FROM rust:1.55-slim@sha256:1ee75015d02a52243217fd9e9e9ca4482ce677dc31e0248d80b4b7ce485a2cbb
RUN apt-get update && apt-get install -y pkg-config libssl-dev && apt-get clean
RUN cargo install cargo-audit
