#!/usr/bin/env bash
set -exuo pipefail

if [ "$#" -ne 1 ]; then
    echo "Script is expecting voteplan index "
	echo "./private.sh 0"
	exit -1
fi

VOTE_PLAN_INDEX=$1
VOTE_PLAN_ID=$(jq -r --arg VOTE_PLAN_INDEX "$VOTE_PLAN_INDEX" '.[$VOTE_PLAN_INDEX|tonumber].id' active_plans.json)
COMMITTEE_KEY=committee_1
COMMITTEE_PK=$(jcli key to-public < "$COMMITTEE_KEY")
MEMBER_SECRET_KEY=$(printf "./%s_committees/%s/member_secret_key.sk" $VOTE_PLAN_ID $COMMITTEE_PK)

jcli "votes" "tally" "decryption-shares" "--vote-plan" "active_plans.json" "--vote-plan-id" "$VOTE_PLAN_ID" "--key"  "$MEMBER_SECRET_KEY" > "$VOTE_PLAN_ID"_decryption_share.json
jcli "votes" "tally" "merge-shares" $VOTE_PLAN_ID"_decryption_share.json" > "$VOTE_PLAN_ID"_shares.json
jcli "votes" "tally" "decrypt-results" "--vote-plan" "active_plans.json" "--vote-plan-id" "$VOTE_PLAN_ID" "--shares" $VOTE_PLAN_ID"_shares.json" "--threshold" "1" "--output-format" "json" > results"$VOTE_PLAN_INDEX".json
