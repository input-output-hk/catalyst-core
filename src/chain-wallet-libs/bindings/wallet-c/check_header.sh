#!/usr/bin/env sh
set -e

# the find command is used mostly so this work from the root directory
# and from the wallet-c directory

EXCLUDE='-not -path "*/target" -not -path "*/.git" -not -path "*/node_modules"'
CONFIG=$(find . -name "cbindgen.toml" $EXCLUDE)
HEADER_FILEPATH=$(find . -name "wallet.h" $EXCLUDE)

# remove this line, as it contains the cbindgen version, causing PR's to fail
# when cbindgen releases a new version.

strip_line_with_version() {
    sed -i '/Generated with cbindgen/d' $1
}

ACTUAL_HEADER=$(mktemp)
GENERATED_HEADER=$(mktemp)

cat $HEADER_FILEPATH >$ACTUAL_HEADER
cbindgen --config $CONFIG --crate jormungandrwallet >$GENERATED_HEADER

strip_line_with_version $ACTUAL_HEADER
strip_line_with_version $GENERATED_HEADER

diff $GENERATED_HEADER $ACTUAL_HEADER

rm "$ACTUAL_HEADER"
rm "$GENERATED_HEADER"
