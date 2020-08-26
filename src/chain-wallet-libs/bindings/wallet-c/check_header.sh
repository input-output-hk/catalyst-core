#!/usr/bin/env bash

# the find command is used mostly so this work from the root directory
# and from the wallet-c directory

EXCLUDE='-not -path "*/target" -not -path "*/.git" -not -path "*/node_modules"'
CONFIG=$(find . -name "cbindgen.toml" $EXCLUDE)
HEADER=$(find . -name "wallet.h" $EXCLUDE)

diff <(cbindgen --config $CONFIG --crate jormungandrwallet) $HEADER
