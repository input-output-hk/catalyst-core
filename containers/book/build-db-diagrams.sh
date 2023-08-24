#!/bin/bash
echo "Starting to build Event DB diagrams."
# doc-event-db-setup
echo "Initializing Event DB."
pushd src/event-db
# [tasks.doc-event-db-init]
while ! psql -e -f setup/setup-db.sql \
    -v dbName=CatalystEventDocs \
    -v dbDescription="Local Docs Catalayst Event DB" \
    -v dbUser="catalyst-event-docs" \
    -v dbUserPw="CHANGE_ME" ${@};
do sleep 1;
done;
# [tasks.run-event-doc-db-migration]
refinery migrate -e DATABASE_URL -c refinery.toml -p ./migrations


## Build the Event DB Documentation (Images of Schema)
popd
echo "Building Event DB diagrams."
# [tasks.build-db-docs-overview-diagram]
dbviz -d CatalystEventDocs \
    -h postgres \
    --title "Catalyst Event Database Overview" \
    -e  funds proposals \
        proposals_voteplans \
        proposal_simple_challenge \
        proposal_community_choice_challenge \
        voteplans \
        api_tokens \
        challenges \
        community_advisors_reviews \
        goals \
        groups \
        full_proposals_info \
        refinery_schema_history \
        > /db-diagrams/event-db-overview.dot
# [tasks.build-db-docs-config-diagram]
dbviz -d CatalystEventDocs \
    -h postgres \
    --title "Catalyst Event Database - Configuration" \
    --comments \
    --column-description-wrap 60 \
    -i  refinery_schema_history \
        config \
        > /db-diagrams/event-db-config.dot
# [tasks.build-db-docs-event-diagram]
dbviz -d CatalystEventDocs \
    -h postgres \
    --title "Catalyst Event Database - Event" \
    --comments \
    --column-description-wrap 140 \
    -i  event \
        goal \
    > /db-diagrams/event-db-event.dot

# [tasks.build-db-docs-objective-proposal-diagram]
dbviz -d CatalystEventDocs \
    -h postgres \
    --title "Catalyst Event Database - Objectives & Proposals" \
    --comments \
    --column-description-wrap 40 \
    -i  challenge_category \
        objective_category \
        currency \
        vote_options \
        objective \
        proposal \
        proposal_review \
        objective_review_metric \
        review_rating \
        review_metric \
        > /db-diagrams/event-db-objective-proposal.dot

# [tasks.build-db-docs-vote-plan-diagram]
dbviz -d CatalystEventDocs \
    -h postgres \
    --title "Catalyst Event Database - Vote Plans" \
    --comments \
    --column-description-wrap 40 \
    -i  voteplan_category \
        voting_group \
        voteplan \
        proposal_voteplan \
        > /db-diagrams/event-db-vote-plan.dot
# [tasks.build-db-docs-snapshot-diagram]
dbviz -d CatalystEventDocs \
    -h postgres \
    --title "Catalyst Event Database - Snapshot" \
    --comments \
    --column-description-wrap 40 \
    -i  snapshot \
        voter \
        contribution \
        ballot \
        voteplan \
        > /db-diagrams/event-db-snapshot-vote.dot
# [tasks.build-db-docs-automation-diagram]
dbviz -d CatalystEventDocs \
    -h postgres \
    --title "Catalyst Event Database - Automation" \
    --comments \
    --column-description-wrap 40 \
    -i  voting_node \
        tally_committee \
        > /db-diagrams/event-db-automation.dot

# [tasks.build-db-docs-moderation-diagram]
dbviz -d CatalystEventDocs \
    -h postgres \
    --title "Catalyst Event Database Moderation Stage" \
    --comments \
    --column-description-wrap 40 \
    -i  moderation_allocation \
        moderation \
        > /db-diagrams/event-db-moderation.dot

echo "Finished building Event DB diagrams."
