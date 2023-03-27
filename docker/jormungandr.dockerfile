# Stage 1: Use base image to build with jormungandr and jcli
FROM catalyst-core-base:latest
RUN cargo install --locked --path src/jormungandr/jormungandr && \
    cargo install --locked --path src/jormungandr/jcli


# Stage 2: Install executables in final image
FROM debian:bullseye-slim
ARG APP_PATH=/app
ARG REST_PORT=8448

## Update container and copy executables
RUN apt-get update && \
    apt-get install -y protobuf-compiler libssl-dev libpq-dev libsqlite3-dev
COPY --from=0 /usr/local/cargo/bin/jormungandr /usr/local/bin/jormungandr
COPY --from=0 /usr/local/cargo/bin/jcli /usr/local/bin/jcli

## apt cleanup
RUN apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR ${APP_PATH}

EXPOSE ${REST_PORT}

CMD [ "/usr/local/bin/jormungandr", "--help" ]
