#! /bin/bash

BASE_URL="https://github.com/input-output-hk/chain-wallet-libs/releases/download/"
VERSION="v0.2.0"
LIST="aarch64-linux-android arm-linux-androideabi armv7-linux-androideabi i686-linux-android x86_64-linux-android"

for target in $LIST; do
    ARCHIVE=chain-wallet-libs-${VERSION}-${target}.tar.gz
    URL="${BASE_URL}/${VERSION}/${ARCHIVE}"
    mkdir -p $target
    pushd $target
    wget ${URL}
    tar xf ${ARCHIVE}
    popd
done