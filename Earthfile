VERSION 0.7

nix:
    FROM debian:stable-slim
    ARG user=user
    ARG uid=1000
    ARG gid=$uid

    # Install Nix dependencies
    RUN apt-get update && apt-get upgrade -y && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        sudo \
        xz-utils

    # Nix doesn't like being run as root, so we create a user to run it
    RUN groupadd -g $gid $user && \
        useradd -u $uid -g $gid -G sudo -m $user -s /bin/bash

    # Setup sudo (used by the installer) and enable flakes
    RUN sed -i 's/%sudo.*ALL/%sudo   ALL=(ALL:ALL) NOPASSWD:ALL/' /etc/sudoers && \
        echo "sandbox = false" > /etc/nix.conf && \
        echo "experimental-features = nix-command flakes" >> /etc/nix.conf

    # Install Nix
    USER $user
    ENV USER=${USER}
    ENV NIX_PATH=/home/${USER}/.nix-defexpr/channels:/nix/var/nix/profiles/per-user/root/channels
    ENV NIX_CONF_DIR /etc
    RUN curl -L 'https://nixos.org/nix/install' | NIX_INSTALLER_NO_MODIFY_PROFILE=1 sh

builder:
    FROM +nix

    ARG user=user
    ENV USER=$user

    # Copy the devshell and dump the environment
    WORKDIR /devshell

    COPY flake.nix flake.lock rust-toolchain .
    COPY --dir nix .
    RUN bash -c "source /home/$user/.nix-profile/etc/profile.d/nix.sh && nix print-dev-env --accept-flake-config >.env"

    # Create a simplified script for executing within the devshell
    RUN echo '#!/usr/bin/env bash' >>with_nix && \
        echo 'source /devshell/.env >/dev/null 2>&1' >>with_nix && \
        echo 'exec "$@"' >>with_nix && \
        chmod +x with_nix && \
        sudo ln -s /devshell/with_nix /usr/bin/with_nix

    WORKDIR /work

    IF [ $EARTHLY_CI == "true" ]
        ARG tag=$(TZ=UTC date +"%Y%m%d%H%M%S")-${EARTHLY_GIT_SHORT_HASH}
    ELSE
        ARG tag=latest
    END

    SAVE IMAGE builder:$tag

install-chef:
    FROM +builder
    RUN with_nix cargo install --debug cargo-chef

prepare-cache:
    FROM +install-chef
    COPY --dir src Cargo.lock Cargo.toml .
    RUN with_nix cargo chef prepare
    SAVE ARTIFACT recipe.json
    SAVE IMAGE --cache-hint

# Using cutoff-optimization to ensure cache hit (see examples/cutoff-optimization)
build-cache:
    FROM +install-chef
    COPY +prepare-cache/recipe.json ./
    RUN with_nix cargo chef cook --release
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home
    SAVE IMAGE --cache-hint

build-workspace:
    FROM +builder
    COPY --dir src Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    SAVE ARTIFACT src

all:
    LOCALLY
    ARG EARTHLY_CI
    ARG EARTHLY_GIT_SHORT_HASH
    ARG registry

    IF [ "$EARTHLY_CI" = "true" ]
        ARG tag=$(TZ=UTC date +"%Y%m%d%H%M%S")-$EARTHLY_GIT_SHORT_HASH
    ELSE
        ARG tag=latest
    END

    IF [ "$registry" = "" ]
        ARG registry_final=$registry
    ELSE
        ARG registry_final=${registry}/
    END

    BUILD ./containers/event-db-migrations+docker --tag=$tag --registry=$registry_final
    BUILD ./src/jormungandr/jormungandr+docker --tag=$tag --registry=$registry_final
    BUILD ./src/jormungandr/jcli+docker --tag=$tag --registry=$registry_final
    BUILD ./src/catalyst-toolbox/catalyst-toolbox+docker --tag=$tag --registry=$registry_final
    BUILD ./src/cat-data-service+docker --tag=$tag --registry=$registry_final
    BUILD ./src/event-db+docker --tag=$tag --registry=$registry_final

ci:
    BUILD ./containers/event-db-migrations+test

test:
    FROM +builder
    RUN with_nix cargo --version
