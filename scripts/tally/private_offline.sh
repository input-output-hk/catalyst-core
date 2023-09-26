#!/usr/bin/env bash
set -exuo pipefail

if [ "$#" -ne 1 ]; then
    echo "Script is expecting voteplan id "
	echo "./private.sh 9a278b6f788278e5cd8dfd6de8b8b8699a7f6b4847c680843de6c02d5b3169b2"
	exit -1
fi

VOTE_PLAN_ID=$1
COMMITTEE_KEY=committee_1
COMMITTEE_PK=$(jcli key to-public < "$COMMITTEE_KEY")
MEMBER_SECRET_KEY=$(printf "./%s_committees/%s/member_secret_key.sk" $VOTE_PLAN_ID $COMMITTEE_PK)

jcli "votes" "tally" "decryption-shares" "--vote-plan" "active_plans.json" "--vote-plan-id" "$VOTE_PLAN_ID" "--key"  "$MEMBER_SECRET_KEY" > "$VOTE_PLAN_ID"_decryption_share.json
jcli "votes" "tally" "merge-shares" $VOTE_PLAN_ID"_decryption_share.json" > "$VOTE_PLAN_ID"_shares.json
jcli "votes" "tally" "decrypt-results" "--vote-plan" "active_plans.json" "--vote-plan-id" "$VOTE_PLAN_ID" "--shares" $VOTE_PLAN_ID"_shares.json" "--threshold" "1" "--output-format" "json" > "$VOTE_PLAN_ID"_result.json
