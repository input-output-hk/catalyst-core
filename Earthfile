# Set the Earthly version to 0.7
VERSION 0.7

# Define the nix stage, which installs Nix and sets up a user to run it
nix:
    FROM debian:stable-slim
    ARG user=user
    ARG uid=1000
    ARG gid=$uid

    # Install Nix dependencies
    RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        sudo \
        xz-utils

    # Create a user to run Nix
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

    SAVE IMAGE --cache-hint

# Define the builder stage, which uses Nix to build the Rust project
builder:
    FROM +nix

    ARG user=user
    ENV USER=$user

    # Copy the devshell and dump the environment
    WORKDIR /devshell

    COPY flake.nix flake.lock rust-toolchain .
    COPY --dir nix .
    RUN bash -c "source /home/$user/.nix-profile/etc/profile.d/nix.sh && nix print-dev-env --accept-flake-config >.env"

    # Add patchelf for patching operations
    RUN bash -c "source /home/$user/.nix-profile/etc/profile.d/nix.sh && nix-env -iA nixpkgs.patchelf"

    # Copy the helper scripts
    WORKDIR /scripts

    COPY scripts/with_nix.sh .
    RUN chmod +x with_nix.sh && \
        sudo ln -s /scripts/with_nix.sh /usr/bin/with_nix

    COPY scripts/collect-libs.sh .
    RUN chmod +x collect-libs.sh && \
        sudo ln -s /scripts/collect-libs.sh /usr/bin/collect-libs

    WORKDIR /work
    SAVE IMAGE --cache-hint

# Define the install-chef stage, which installs the cargo-chef tool
install-chef:
    FROM +builder
    RUN with_nix cargo install --debug cargo-chef

# Define the prepare-cache stage, which prepares the build cache
prepare-cache:
    FROM +install-chef
    COPY --dir src Cargo.lock Cargo.toml .
    RUN with_nix cargo chef prepare
    SAVE ARTIFACT recipe.json
    SAVE IMAGE --cache-hint

# Define the build-cache stage, which builds the cache
build-cache:
    FROM +install-chef
    COPY +prepare-cache/recipe.json ./
    RUN with_nix cargo chef cook --release
    SAVE ARTIFACT target
    SAVE ARTIFACT $CARGO_HOME cargo_home
    SAVE IMAGE --cache-hint

# Define the build-workspace
build-workspace:
    FROM +builder
    COPY --dir src Cargo.lock Cargo.toml .
    COPY +build-cache/cargo_home $CARGO_HOME
    COPY +build-cache/target target
    SAVE ARTIFACT src

# Define the all stage, which builds and tags all Docker images
all:
    LOCALLY
    ARG EARTHLY_CI
    ARG EARTHLY_GIT_SHORT_HASH
    ARG registry

    # Set the tag for the Docker image
    IF [ "$EARTHLY_CI" = "true" ]
        ARG tag=$(TZ=UTC date +"%Y%m%d%H%M%S")-$EARTHLY_GIT_SHORT_HASH
    ELSE
        ARG tag=latest
    END

    # Determine the final registry to push to
    IF [ "$registry" = "" ]
        ARG registry_final=$registry
    ELSE
        ARG registry_final=${registry}/
    END

    # Build and tag all Docker images
    BUILD ./containers/event-db-migrations+docker --tag=$tag --registry=$registry_final
    BUILD ./src/jormungandr/jormungandr+docker --tag=$tag --registry=$registry_final
    BUILD ./src/jormungandr/jcli+docker --tag=$tag --registry=$registry_final
    BUILD ./src/catalyst-toolbox/catalyst-toolbox+docker --tag=$tag --registry=$registry_final
    BUILD ./src/cat-data-service+docker --tag=$tag --registry=$registry_final
    BUILD ./src/event-db+docker --tag=$tag --registry=$registry_final

# Define the ci stage, which only builds the event-db-migrations Docker image for testing
ci:
    BUILD ./containers/event-db-migrations+test

# Define the test stage, which runs the Rust project's tests
test:
    FROM +builder
    RUN with_nix cargo --version
