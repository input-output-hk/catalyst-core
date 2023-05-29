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

# Define the command to be executed
ARGS=$*
CMD="/app/snapshot_tool $ARGS"
echo ">>> Executing command: $CMD"

# Wait for DEBUG_SLEEP seconds if the DEBUG_SLEEP environment variable is set
if [ -n "${DEBUG_SLEEP:-}" ]; then
  echo "DEBUG_SLEEP is set to $DEBUG_SLEEP. Sleeping..."
  sleep "$DEBUG_SLEEP"
fi

# Expand the command with arguments and capture the exit code
set +e
eval "$CMD"
EXIT_CODE=$?
set -e

# If the exit code is 0, the executable returned successfully
if [ $EXIT_CODE -ne 0 ]; then
  echo "Error: snapshot_tool returned with exit code $EXIT_CODE"
  exit 1
fi
