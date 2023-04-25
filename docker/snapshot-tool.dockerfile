# Stage 1: Use base image to build snapshot_tool
FROM catalyst-core-base:latest
RUN cargo install --locked --path src/voting-tools-rs


# Stage 2: Install executables in final image
FROM debian:bullseye-slim
ARG APP_PATH=/app

## Update container and copy executables
RUN apt-get update && \
    apt-get install -y protobuf-compiler libssl-dev libpq-dev libsqlite3-dev
COPY --from=0 /usr/local/cargo/bin/snapshot_tool /usr/local/bin/snapshot_tool

## cleanup
RUN apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR ${APP_PATH}

CMD [ "/usr/local/bin/snapshot_tool", "--help" ]
