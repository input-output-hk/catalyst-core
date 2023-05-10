#!/usr/bin/env bash

# shellcheck disable=SC1091
source /devshell/.env >/dev/null 2>&1
source "$HOME/.nix-profile/etc/profile.d/nix.sh" 2>&1
exec "$@"
