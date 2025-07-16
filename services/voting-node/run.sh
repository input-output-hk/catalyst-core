#!/usr/bin/env bash

set -e

echo ">>> Fetching secrets"

DB_SECRET=$(aws secretsmanager get-secret-value --secret-id dev/db/eventdb | jq -r .SecretString)
EXTERNAL_SECRET=$(aws secretsmanager get-secret-value --secret-id dev/voting-node/external | jq -r .SecretString)
GEN_SECRET=$(aws secretsmanager get-secret-value --secret-id dev/voting-node/generated | jq -r .SecretString)
MAINNET_SECRET=$(aws secretsmanager get-secret-value --secret-id global/db/dbsync-mainnet | jq -r .SecretString)
PREPROD_SECRET=$(aws secretsmanager get-secret-value --secret-id global/db/dbsync-preprod | jq -r .SecretString)
SSH_SECRET=$(aws secretsmanager get-secret-value --secret-id global/db/dbsync-ssh | jq -r .SecretString)

echo ">>> Setting up env variables"

DB_USERNAME=$(echo "${DB_SECRET}" | jq -r '.username')
DB_PASSWORD=$(echo "${DB_SECRET}" | jq -r '.password')
DB_HOST=$(echo "${DB_SECRET}" | jq -r '.host')
DB_PORT=$(echo "${DB_SECRET}" | jq -r '.port')
DB_URL="postgres://$DB_USERNAME:$DB_PASSWORD@$DB_HOST:$DB_PORT/eventdb"

EXTERNAL_URL=$(echo "${EXTERNAL_SECRET}" | jq -r '.ideascale_url')
EXTERNAL_TOKEN=$(echo "${EXTERNAL_SECRET}" | jq -r '.ideascale_token')

GEN_CRS=$(echo "${GEN_SECRET}" | jq -r '.crs')
GEN_SECRET_SECRET=$(echo "${GEN_SECRET}" | jq -r '.secret')

MAINNET_USERNAME=$(echo "${MAINNET_SECRET}" | jq -r '.username')
MAINNET_PASSWORD=$(echo "${MAINNET_SECRET}" | jq -r '.password')
MAINNET_HOST=$(echo "${MAINNET_SECRET}" | jq -r '.host')
MAINNET_PORT=$(echo "${MAINNET_SECRET}" | jq -r '.port')
MAINNET_URL="postgres://${MAINNET_USERNAME}:${MAINNET_PASSWORD}@${MAINNET_HOST}:${MAINNET_PORT}/dbsync-mainnet"

PREPROD_USERNAME=$(echo "${PREPROD_SECRET}" | jq -r '.username')
PREPROD_PASSWORD=$(echo "${PREPROD_SECRET}" | jq -r '.password')
PREPROD_HOST=$(echo "${PREPROD_SECRET}" | jq -r '.host')
PREPROD_PORT=$(echo "${PREPROD_SECRET}" | jq -r '.port')
PREPROD_URL="postgres://${PREPROD_USERNAME}:${PREPROD_PASSWORD}@${PREPROD_HOST}:${PREPROD_PORT}/dbsync-preprod"

SSH_PRIVATE_KEY=$(echo "${SSH_SECRET}" | jq -r '.private')
SSH_PUBLIC_KEY=$(echo "${SSH_SECRET}" | jq -r '.public')
SSH_HOST_KEY=$(echo "${SSH_SECRET}" | jq -r '."host-key"')
SSH_USER=$(echo "${SSH_SECRET}" | jq -r '.user')
SSH_HOST=$(echo "${SSH_SECRET}" | jq -r '.host')
SSH_ADDRESS="${SSH_USER}@${SSH_HOST}"

echo ">>> Running the voting node"

docker run -d \
    --hostname leader0 \
    -p 8080:8080 \
    -v "$(pwd)/ideascale.json:/configs/ideascale.json" \
    -e TESTNET_DBSYNC_URL="${PREPROD_URL}" \
    -e MAINNET_DBSYNC_URL="${MAINNET_URL}" \
    -e EVENTDB_URL="${DB_URL}" \
    -e COMMITTEE_CRS="${GEN_CRS}" \
    -e SECRET_SECRET="${GEN_SECRET_SECRET}" \
    -e DBSYNC_SSH_PRIVKEY="${SSH_PRIVATE_KEY}" \
    -e DBSYNC_SSH_PUBKEY="${SSH_PUBLIC_KEY}" \
    -e DBSYNC_SSH_HOST_KEY="${SSH_HOST_KEY}" \
    -e SSH_SNAPSHOT_TOOL_DESTINATION="${SSH_ADDRESS}" \
    -e IDEASCALE_API_URL="${EXTERNAL_URL}" \
    -e IDEASCALE_API_TOKEN="${EXTERNAL_TOKEN}" \
    -e SNAPSHOT_TOOL_SSH="true" \
    -e SSH_SNAPSHOT_TOOL_PATH="/home/snapshot/.local/bin/snapshot_tool" \
    -e SSH_SNAPSHOT_TOOL_OUTPUT_DIR="/home/snapshot/dev_snapshot_tool_out" \
    -e GVC_API_URL="unused" \
    -e IS_NODE_RELOADABLE="true" \
    -e VOTING_HOST="0.0.0.0" \
    -e VOTING_PORT="8080" \
    -e VOTING_LOG_LEVEL="debug" \
    -e JORM_PATH="jormungandr" \
    -e JCLI_PATH="jcli" \
    -e IDEASCALE_CONFIG_PATH="/configs/ideascale.json" \
    -e IDEASCALE_CAMPAIGN_GROUP="66" \
    -e IDEASCALE_STAGE_ID="4385" \
    -e IDEASCALE_LOG_LEVEL="debug" \
    -e IDEASCALE_LOG_FORMAT="text" \
    -e SNAPSHOT_INTERVAL_SECONDS="3600" \
    -e SNAPSHOT_CONFIG_PATH="/app/snapshot-importer-example-config.json" \
    -e SNAPSHOT_OUTPUT_DIR="/tmp/snapshot-output" \
    -e SNAPSHOT_NETWORK_IDS="testnet" \
    -e SNAPSHOT_LOG_LEVEL="debug" \
    -e SNAPSHOT_LOG_FORMAT="text" \
    voting-node:latest
