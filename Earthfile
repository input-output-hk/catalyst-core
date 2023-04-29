VERSION 0.6
FROM rust:1.65

ARG EARTHLY_CI
IF [ $EARTHLY_CI == "true" ]
    ARG tag=$(TZ=UTC date +"%Y%m%d%H%M%S")-${EARTHLY_GIT_SHORT_HASH}
ELSE
    ARG tag=latest
END

ARG registry
IF [ ! -z $registry && "$(echo "$registry" | tail -c 2)" != "/" ]
    ARG registry_final=${registry}/
ELSE
    ARG registry_final=$registry
END

build-rust:
    WORKDIR /catalyst-core
    RUN rustup component add rustfmt

install-chef:
    FROM +build-rust
    RUN cargo install --debug cargo-chef

prepare-cache:
    FROM +install-chef
    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo chef prepare
    SAVE ARTIFACT recipe.json

# Using cutoff-optimization to ensure cache hit (see examples/cutoff-optimization)
build-cache:
    FROM +install-chef
    RUN apt-get update && \
        apt-get install -y protobuf-compiler libssl-dev libpq-dev libsqlite3-dev
    COPY +prepare-cache/recipe.json ./
    RUN cargo chef cook --release
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home

build-workspace:
    FROM +build-rust
    COPY --dir src Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    SAVE ARTIFACT src

all:
    BUILD ./containers/event-db-migrations+docker --tag=$tag --registry=$registry_final
    # BUILD ./src/jormungandr/jormungandr+docker --tag=$tag --registry=$registry_final
    # BUILD ./src/jormungandr/jcli+docker --tag=$tag --registry=$registry_final
    # BUILD ./src/catalyst-toolbox/catalyst-toolbox+docker --tag=$tag --registry=$registry_final
    # BUILD ./src/cat-data-service+docker --tag=$tag --registry=$registry_final
    # BUILD ./src/event-db+docker --tag=$tag --registry=$registry_final

ci:
    BUILD ./containers/event-db-migrations+test
