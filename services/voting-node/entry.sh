#!/bin/bash

# ---------------------------------------------------------------
# Entrypoint script for voting-node container
# ---------------------------------------------------------------
#
# This script serves as the entrypoint for the voting-node container. It sets up
# the environment, and then runs the voting node.
#
# It expects the following environment variables to be set except where noted:
#
# EVENTDB_URL - The URL of the event database
# IS_NODE_RELOADABLE - If set, the voting node will reload its configuration (optional). Defaults to true
# VOTING_HOST - Voting node IP (optional). Defaults to 0.0.0.0
# VOTING_PORT - Voting node port (optional). Defaults to 8000
# VOTING_LOG_LEVEL - Log level (optional). Defaults to info
# VOTING_LOG_FORMAT - Log format (optional). Defaults to text
# VOTING_NODE_STORAGE - Path to node storage (optional). Defaults to ./node_storage
# JORM_PATH - Path to jormungandr executable (optional). Defaults to jormungandr
# JCLI_PATH - Path to jcli executable (optional). Defaults to jcli
#
# For the case that the hostname is 'leader0', the following environment variables must be set:
#
# ### SECRET GENERATION
# COMMITTEE_CRS - The CRS is used to generate committee members, this is only used by leader0
# SECRET_SECRET - The password used to encrypt/decrypt secrets in the database
#
# ### IDEASCALE DATA IMPORTER
# IDEASCALE_API_TOKEN - API token for IDEASCALE
# IDEASCALE_API_URL - URL for IdeaScale. Example: https://cardano.ideascale.com
#
# ### DBSYNC SNAPSHOT DATA IMPORTER
# SNAPSHOT_TOOL_PATH - Path to snapshot tool executable (optional). Defaults to 'snapshot_tool'
# CATALYST_TOOLBOX_PATH - Path to toolbox executable (optional). Defaults to 'catalyst-toolbox'
# GVC_API_URL - URL for GVC
# SNAPSHOT_OUTPUT_DIR - Path to directory where snapshot data will be stored
# SNAPSHOT_NETWORK_IDS - Network IDs (separated by space) for snapshot data. Possible values are 'mainnet' and 'testnet'.
# SNAPSHOT_INTERVAL_SECONDS - Interval in seconds for snapshot data (optional)
# ---------------------------------------------------------------

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
    "DBSYNC_SSH_HOST_KEY"
    "DBSYNC_SSH_PRIVKEY"
    "DBSYNC_SSH_PUBKEY"
    "EVENTDB_URL"
)
echo ">>> Checking required env vars..."
check_env_vars "${REQUIRED_ENV[@]}"

: "${IS_NODE_RELOADABLE:='true'}"
: "${VOTING_HOST:='0.0.0.0'}"
: "${VOTING_PORT:='8000'}"
: "${VOTING_NODE_STORAGE:='./node_storage'}"
: "${JORM_PATH:='jormungandr'}"
: "${JCLI_PATH:='jcli'}"

# Export environment variables
export IS_NODE_RELOADABLE="${IS_NODE_RELOADABLE}"
export VOTING_HOST="${VOTING_HOST}"
export VOTING_PORT="${VOTING_PORT}"
export VOTING_NODE_STORAGE="${VOTING_NODE_STORAGE}"
export JORM_PATH="${JORM_PATH}"
export JCLI_PATH="${JCLI_PATH}"

# Get the hostname
HOSTNAME=$(hostname)

# Check if the hostname is 'leader0'
if [ "$HOSTNAME" = "leader0" ]; then
    echo ">>> 'leader0' found. Checking required env vars..."
    LEADER0_ENV=(
        "COMMITTEE_CRS"
        "SECRET_SECRET"
        "IDEASCALE_API_TOKEN"
        "IDEASCALE_API_URL"
        #"TESTNET_DBSYNC_URL"
        #"MAINNET_DBSYNC_URL"
        "GVC_API_URL"
        "SNAPSHOT_OUTPUT_DIR"
        "SNAPSHOT_NETWORK_IDS"
    )
    check_env_vars "${LEADER0_ENV[@]}"

    : "${SNAPSHOT_TOOL_PATH:='snapshot_tool'}"
    : "${CATALYST_TOOLBOX_PATH:='catalyst-toolbox'}"
    : "${SNAPSHOT_INTERVAL_SECONDS:='1800'}"

    # Export environment variables
    export SNAPSHOT_TOOL_PATH="${SNAPSHOT_TOOL_PATH}"
    export CATALYST_TOOLBOX_PATH="${CATALYST_TOOLBOX_PATH}"
    export SNAPSHOT_INTERVAL_SECONDS="${SNAPSHOT_INTERVAL_SECONDS}"
fi

# Setup dbsync SSH keys
echo ">>> Setting up dbsync SSH keys..."
mkdir -p /root/.ssh
echo -n "${DBSYNC_SSH_PRIVKEY}" | base64 -d >/root/.ssh/id_snapshot
echo -n "${DBSYNC_SSH_PUBKEY}" | base64 -d >/root/.ssh/id_snapshot.pub
echo -n "${DBSYNC_SSH_HOST_KEY}" | base64 -d >/root/.ssh/known_hosts
chmod 0700 /root/.ssh
chmod 0600 /root/.ssh/*

export SSH_SNAPSHOT_TOOL_KEYFILE=/root/.ssh/id_snapshot

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
