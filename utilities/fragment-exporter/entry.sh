#!/usr/bin/env bash

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
  "LEADER_IP_PREFIX"
  "LEADER_PORT"
  "FRAGMENT_EXPORTER_INDEX_PATH"
)
echo ">>> Checking required env vars..."
check_env_vars "${REQUIRED_ENV[@]}"

# Calculate the correct leader IP address (e.g. <prefix>.<index>)
HOSTNAME=$(hostname)
export FRAGMENT_EXPORTER_URL="http://${LEADER_IP_PREFIX}.${HOSTNAME##*-}:${LEADER_PORT}"

# Print out the environment variables
echo ">>> Using FRAGMENT_EXPORTER_URL: ${FRAGMENT_EXPORTER_URL}"
echo ">>> Using FRAGMENT_EXPORTER_INDEX_PATH: ${FRAGMENT_EXPORTER_INDEX_PATH}"

# Define the command to be executed
CMD_TO_RUN="fragment_exporter"

# Add $* to the command so that additional flags can be passed
ARGS="$*"
CMD="$CMD_TO_RUN $ARGS"

# Expand the command with arguments and capture the exit code
eval "$CMD"
