# Dockerfile example to build a container with jormungandr and jcli
FROM catalyst-core-base:latest
RUN cargo install --locked --path src/jormungandr/jormungandr && \
    cargo install --locked --path src/jormungandr/jcli


FROM debian:bullseye-slim
LABEL MAINTAINER IOHK
LABEL description="Jormungandr latest"

ARG APP_PATH=/app
ARG REST_PORT=8448

# Update container and copy executables
RUN apt-get update && \
    apt-get install -y curl git build-essential pkg-config \
                       protobuf-compiler libssl-dev libpq-dev libsqlite3-dev
COPY --from=0 /usr/local/cargo/bin/jormungandr /usr/local/bin/jormungandr
COPY --from=0 /usr/local/cargo/bin/jcli /usr/local/bin/jcli

# cleanup
RUN apt-get remove --purge --auto-remove -y git curl build-essential pkg-config && \
    apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR ${APP_PATH}
# TODO: Add files to this path needed for the node
EXPOSE ${REST_PORT}

CMD [ "/usr/local/bin/jormungandr", "--help" ]
