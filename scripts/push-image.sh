#!/usr/bin/env bash

set -o errexit -o pipefail -o noclobber -o nounset

help() {
    echo "Usage: $(basename "$0") [-hl] image_name"
    echo
    echo "   -h           show this help message"
    echo "   -l           list all image names"
}

list_images() {
    system="$(nix eval --raw --impure --expr 'builtins.currentSystem')"
    nix eval --json ".#containers.$system" --apply builtins.attrNames | jq -r '.[]'
}

push_image() {
    system="$(nix eval --raw --impure --expr 'builtins.currentSystem')"
    nix run ".#containers.$system.$1.copyToRegistry"
}

while getopts ':hl' option; do
    case "$option" in
    h)
        help
        exit
        ;;
    l)
        list_images
        exit
        ;;
    \?)
        printf "illegal option: -%s\n" "$OPTARG" >&2
        help
        exit 1
        ;;
    esac
done

shift $((OPTIND - 1))

if [[ $# -ne 1 ]]; then
    help
    exit 1
fi

push_image "$1"
