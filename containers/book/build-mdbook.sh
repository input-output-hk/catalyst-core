#!/bin/bash

# Enable strict mode
set +x
set -o errexit
set -o pipefail
set -o nounset
set -o functrace
set -o errtrace
set -o monitor
set -o posix
shopt -s dotglob

echo ">>> Building mdbook..."
CMD="mdbook build"
# will retry to build 6 times every 5 seconds
if  ! (r=6; while ! eval "$CMD" ; do
    ((--r))||exit
    echo "Build failed. Retrying.";
    sleep 5;done) ; then
    exit 1
fi

echo ">>> Build successful.";
exit 0
