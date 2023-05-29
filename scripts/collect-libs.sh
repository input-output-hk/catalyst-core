#!/usr/bin/env bash

# collect-libs.sh
#
# Description:
#   This script copies the shared libraries required by a binary to a
#   destination directory.
#
# Usage:
#   collect-libs.sh <binary_path> <outout_path>
#
# Arguments:
#   binary_path: The path to the binary.
#   output_path: The path to the directory where the shared libraries will be
#                copied.
#
# Requirements:
#   - jq
#   - pylddwrap

set +x
set -o errexit
set -o pipefail

if [[ -z "$1" ]]; then
    echo "usage: $0 <binary_path> <outout_path>" 1>&2
    exit 1
fi

if [[ -z "$2" ]]; then
    echo "usage: $0 <binary_path> <outout_path>" 1>&2
    exit 1
fi

binary_path=$1
output_path=$2

if [[ ! -f "$binary_path" ]]; then
    echo "error: $binary_path does not exist"
    exit 1
fi

if [[ ! -d "$output_path" ]]; then
    echo "error: $output_path does not exist"
    exit 1
fi

ldd_json=$(pylddwrap -f json "$binary_path")
libs=$(echo "$ldd_json" | jq -r '.[] | select(.path != "None") | select(.path | startswith("/nix/store")) | .path')

for lib in $libs; do
    echo "copying $lib to $output_path"
    cp "$lib" "$output_path"
done

# Special handling for the dynamic linker/loader
ld_linux_so=$(echo "$ldd_json" | jq -r '.[] | select(.path != "None") | select(.soname | startswith("/nix/store")) | .soname')
if [[ -n "$ld_linux_so" ]]; then
    echo "copying $ld_linux_so to $output_path"
    cp "$ld_linux_so" "$output_path"
fi
