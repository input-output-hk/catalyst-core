# Stage 1: Use base image to build vit-servicing-station-server
FROM catalyst-core-base:latest
RUN cargo install --locked --path src/vit-servicing-station/vit-servicing-station-server


# Stage 2: Install executables in final image
FROM debian:bullseye-slim
ARG APP_PATH=/app

## Update container and copy executables
RUN apt-get update && \
    apt-get install -y protobuf-compiler libssl-dev libpq-dev libsqlite3-dev
COPY --from=0 /usr/local/cargo/bin/vit-servicing-station-server /usr/local/bin/vit-servicing-station-server

## cleanup
RUN apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR ${APP_PATH}

CMD [ "/usr/local/bin/vit-servicing-station-server", "--help" ]
