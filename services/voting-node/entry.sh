#!/bin/bash
# This script is meant to be an entrypoint for a container image, but can also be used locally.

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

check_env_vars() {
    local env_vars=("$@")

    # Iterate over the array and check if each variable is set
    for var in "${env_vars[@]}"; do
        echo "Checking $var"
        if [ -z "${!var}" ]; then
            echo ">>> Error: $var is required and not set."
            exit 1
        fi
    done
}

debug_sleep() {
    if [ -n "${DEBUG_SLEEP:-}" ]; then
        echo "DEBUG_SLEEP is set. Sleeping for ${DEBUG_SLEEP} seconds..."
        sleep "${DEBUG_SLEEP}"
    fi
}

echo ">>> Starting entrypoint script..."

# Check if all required environment variables are set
REQUIRED_ENV=(
    "IS_NODE_RELOADABLE"
    "VOTING_HOST"
    "VOTING_PORT"
    "VOTING_LOG_LEVEL"
    "VOTING_NODE_STORAGE"
    "EVENTDB_URL"
    "JORM_PATH"
    "JCLI_PATH"
)
echo ">>> Checking required env vars..."
check_env_vars "${REQUIRED_ENV[@]}"

# Get the hostname
HOSTNAME=$(hostname)

# Check if the hostname is 'leader0'
if [ "$HOSTNAME" = "leader0" ]; then
    echo ">>> 'leader0' found. Checking required env vars..."
    LEADER0_ENV=(
        "COMMITTEE_CRS"
        "SECRET_SECRET"
        "IDEASCALE_API_TOKEN"
        "IDEASCALE_CAMPAIGN_GROUP"
        "IDEASCALE_STAGE_ID"
        "IDEASCALE_LOG_LEVEL"
        "IDEASCALE_API_URL"
        "SNAPSHOT_CONFIG_PATH"
        "SNAPSHOT_OUTPUT_DIR"
        "SNAPSHOT_NETWORK_ID"
    )
    check_env_vars "${LEADER0_ENV[@]}"
fi

# Sleep if DEBUG_SLEEP is set
debug_sleep

# Define the command to be executed
CMD_TO_RUN="voting-node start"

# Add $* to the command so that additional flags can be passed
ARGS=$*
CMD="$CMD_TO_RUN $ARGS"

# Wait for DEBUG_SLEEP seconds if the DEBUG_SLEEP environment variable is set
if [ -n "${DEBUG_SLEEP:-}" ]; then
  echo "DEBUG_SLEEP is set to ${DEBUG_SLEEP}. Sleeping..."
  sleep "$DEBUG_SLEEP"
fi

echo ">>> Executing command..."
# Expand the command with arguments and capture the exit code
set +e
eval "$CMD"
EXIT_CODE=$?
set -e

# If the exit code is 0, the Python executable returned successfully
if [ $EXIT_CODE -ne 0 ]; then
  echo "Error: Python executable returned with exit code $EXIT_CODE"
  exit 1
fi
