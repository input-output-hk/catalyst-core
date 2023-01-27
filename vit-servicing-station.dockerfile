# Dockerfile example to build a container with vit-servicing-station-server
FROM catalyst-core-base:latest
RUN cargo install --locked --path src/vit-servicing-station/vit-servicing-station-server


FROM debian:bullseye-slim
LABEL MAINTAINER IOHK
LABEL description="Vit servicing station server"

ARG APP_PATH=/app
    
# Update container and copy executables
RUN apt-get update && \
    apt-get install -y curl git build-essential pkg-config \
                       protobuf-compiler libssl-dev libpq-dev libsqlite3-dev
COPY --from=0 /usr/local/cargo/bin/vit-servicing-station-server /usr/local/bin/vit-servicing-station-server

# cleanup
RUN apt-get remove --purge --auto-remove -y git curl build-essential pkg-config && \
    apt-get install -y --no-install-recommends && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR ${APP_PATH}
# TODO: Add files to this path needed for the node

CMD [ "/usr/local/bin/vit-servicing-station-server", "--help" ]
