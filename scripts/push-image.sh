#!/usr/bin/env bash

set -o errexit -o pipefail -o noclobber -o nounset

ECR_REGISTRY="332405224602.dkr.ecr.eu-central-1.amazonaws.com"

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

write_image_hash() {
    system="$(nix eval --raw --impure --expr 'builtins.currentSystem')"
    store=$(nix eval --json ".#containers.$system" | jq -r --arg name "$1" '.[$name]')
    hash=$(echo "$store" | cut -d '-' -f 1 | cut -d '/' -f 4)

    echo ">>> The image is available at the following location:"
    echo "$ECR_REGISTRY/$1:$hash"
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

echo ">>> Pushing image $1 to $ECR_REGISTRY"
push_image "$1"

echo ">>> Done!"
write_image_hash "$1"
