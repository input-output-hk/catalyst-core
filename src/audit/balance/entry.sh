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

echo ">>> Entering entrypoint script..."
# Verify the storage path exists
if [[ ! -d "$STORAGE_PATH" ]]; then
    echo "ERROR: storage path does not exist at: $STORAGE_PATH";
    echo ">>> Aborting..."
    exit 1
fi
# Verify config is present
if [[ ! -f "$NODE_CONFIG_PATH" ]]; then
    echo "ERROR: node configuration is absent at: $NODE_CONFIG_PATH"
    echo ">>> Aborting..."TH"
    exit 1
fi
# Verify genesis block is present
if [[ ! -f "$GENESIS_PATH" ]]; then
    echo "ERROR: genesis block is absent at: $GENESIS_PA
    echo ">>> Aborting..."
    exit 1
fi
# Allow overriding jormungandr binary
if [[ ! -f "$BIN_PATH" ]]; then
    echo "ERROR: path to jormungandr binary is absent at: $BIN_PATH"
    echo ">>> Aborting..."
    exit 1
fi
echo ">>> Using the following parameters:"
echo "Storage path: $STORAGE_PATH"
echo "Node config: $NODE_CONFIG_PATH"
echo "Genesis block: $GENESIS_PATH"
echo "Binary path: $BIN_PATH"
args=()
args+=("--storage" "$STORAGE_PATH")
args+=("--config" "$NODE_CONFIG_PATH")
args+=("--genesis-block" "$GENESIS_PATH")

# Define the command to be executed
ARGS="${args[*]}"
EXTRA_ARGS=$*
CMD="$BIN_PATH $ARGS $EXTRA_ARGS"
echo ">>> Executing command: $CMD"

# Wait for DEBUG_SLEEP seconds if the DEBUG_SLEEP environment variable is set
if [ -n "${DEBUG_SLEEP:-}" ]; then
  echo "DEBUG_SLEEP is set to $DEBUG_SLEEP. Sleeping..."
  sleep "$DEBUG_SLEEP"
fi

echo "Starting node..."
# Expand the command with arguments and capture the exit code
set +e
eval "$CMD"
EXIT_CODE=$?
set -e

# If the exit code is 0, the executable returned successfully
if [ $EXIT_CODE -ne 0 ]; then
  echo "Error: jormungandr returned with exit code $EXIT_CODE"
  exit 1
fi

