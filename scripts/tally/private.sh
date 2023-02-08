#!/usr/bin/env bash
set -exuo pipefail

export JORMUNGANDR_RESTAPI_URL=https://servicing-station.gov.iog.io/api

if [ "$#" -ne 2 ]; then
    echo "Script is expecting voteplan index and expiry block date: "
	echo "./private.sh 0 1.0"
	exit -1
fi

VOTE_PLAN_INDEX=$1
EXPIRY_BLOCK_DATE=$2

VOTE_PLAN_ID=$(jcli rest v0 vote active plans get --output-format json|jq -r --arg VOTE_PLAN_INDEX "$VOTE_PLAN_INDEX" '.[$VOTE_PLAN_INDEX|tonumber].id')

COMMITTEE_KEY=committee_1
COMMITTEE_PK=$(jcli key to-public < "$COMMITTEE_KEY")
COMMITTEE_ADDR=$(jcli address account "$(jcli key to-public < "$COMMITTEE_KEY")")
COMMITTEE_ADDR_COUNTER=$(jcli rest v0 account get "$COMMITTEE_ADDR" --output-format json|jq -r .counter)
MEMBER_SECRET_KEY=$(printf "./%s_committees/%s/member_secret_key.sk" $VOTE_PLAN_ID $COMMITTEE_PK)

curl $JORMUNGANDR_RESTAPI_URL/v0/block0 > block0.bin

BLOCK0_HASH=$(jcli genesis hash --input block0.bin)

jcli rest v0 vote active plans get --output-format json > active_plans.json
COMMITTEE_ADDR_COUNTER=$(jcli rest v0 account get "$COMMITTEE_ADDR" --output-format json|jq -r .counters[0])

jcli "votes" "tally" "decryption-shares" "--vote-plan" "active_plans.json" "--vote-plan-id" "$VOTE_PLAN_ID" "--key"  "$MEMBER_SECRET_KEY" > decryption_share.json

jcli "votes" "tally" "merge-shares" "decryption_share.json" > shares.json

jcli "votes" "tally" "decrypt-results" "--vote-plan" "active_plans.json" "--vote-plan-id" "$VOTE_PLAN_ID" "--shares" "shares.json" "--threshold" "1" "--output-format" "json" > result.json

jcli "certificate" "new" "vote-tally" "private" "--shares" "shares.json" "--vote-plan" "result.json" "--vote-plan-id" "$VOTE_PLAN_ID" --output "vote-tally.certificate"
jcli "transaction" "new" "--staging" "transaction.tx"
jcli "transaction" "add-account" "$COMMITTEE_ADDR" "0" "--staging" "transaction.tx"
jcli "transaction" "set-expiry-date" "$EXPIRY_BLOCK_DATE" "--staging" "transaction.tx"
jcli "transaction" "add-certificate" "$(< vote-tally.certificate)" "--staging" "transaction.tx"
jcli "transaction" "finalize" "--staging" "transaction.tx"
jcli "transaction" "data-for-witness" "--staging" "transaction.tx" > vote-tally.witness_data
jcli "transaction" "make-witness" "--genesis-block-hash" "$BLOCK0_HASH" "--type" "account" "--account-spending-counter" "$COMMITTEE_ADDR_COUNTER" "--account-spending-counter-lane" "0" $(< vote-tally.witness_data) vote_tally.witness "$COMMITTEE_KEY"
jcli "transaction" "add-witness" "vote_tally.witness" "--staging" "transaction.tx"
jcli "transaction" "seal" "--staging" "transaction.tx"
jcli "transaction" "auth" "--staging" "transaction.tx" "--key" "$COMMITTEE_KEY"
jcli "transaction" "to-message" "--staging" "transaction.tx" > vote-tally.fragment
jcli rest v0 message post --file vote-tally.fragment
