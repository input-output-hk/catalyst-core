# Seeding Development Data

This document describes how to seed the development database with data for testing and development purposes.

## Prerequisites

## 1. Database Migrations

The database migrations must be run before seeding the database.
In order to run the migrations, you must have the `refinery` CLI installed. You can install it by running:

```bash
cargo install refinery_cli
```

You may need to edit [refinery.toml](../../../../../src/event-db/refinery.toml) to match your database configuration.
Then, run the migrations (this assumes you are at the root of the repository):

```bash
cd src/event-db
refinery migrate
```

## Seeding

### 1. Event, Voteplan and Voting Groups

```sql
INSERT INTO event (
    name,
    description,
    voting_power_threshold,
    max_voting_power_pct,
    committee_size,
    committee_threshold,
    start_time,
    end_time,
    registration_snapshot_time,
    insight_sharing_start,
    proposal_submission_start,
    refine_proposals_start,
    finalize_proposals_start,
    proposal_assessment_start,
    assessment_qa_start,
    snapshot_start,
    voting_start,
    voting_end,
    tallying_end,
    extra
) VALUES (
    'Fund 10',
    'This is a test event for Fund 10',
    10,
    6,
    500,
    0.01,
    '2023-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '2021-03-24 00:00:00',
    '{"url": {"results": "https://event.com/results/10", "survey": "https://event.com/survey/10"}}'
) RETURNING row_id;
```

Let EVENT_ROW_ID be the `row_id` of the event that was just inserted:

```sql
INSERT INTO voting_group (
    group_id,
    event_id,
    token_id
) VALUES (
    'group-id-1',
    EVENT_ROW_ID,
    'token-id-1'
) RETURNING row_id;
```

Let GROUP_ROW_ID be the `row_id` of the voting group that was just inserted:

```sql
INSERT INTO voteplan (
    event_id,
    id,
    category,
    encryption_key,
    group_id
) VALUES (
    EVENT_ROW_ID,
    'voteplan-id-1',
    'public',
    'encryption-key-1',
    GROUP_ROW_ID
) RETURNING row_id;
```

### 2. Challenge

```sql
INSERT INTO challenge (
    id,
    event,
    category,
    title,
    description,
    rewards_currency,
    rewards_total,
    proposers_rewards,
    vote_options,
    extra
) VALUES (
    1,
    EVENT_ROW_ID,
    'simple',
    'Challenge Title',
    'Challenge Description',
    'ADA',
    1000000,
    100000,
    (SELECT id FROM vote_options WHERE challenge = 'yes,no'),
    '{"url": {"challenge": "https://challenge.com/1"},"highlights": {"sponsor": "Highlight sponsor 1"}}'
) RETURNING row_id;
```

### 3. Proposals and Proposal Voteplan

Here, `CHALLENGE_ID` is the `id` of the challenge that was just inserted.

```sql
INSERT INTO proposal (
    id,
    challenge,
    title,
    summary,
    category,
    public_key,
    funds,
    url,
    files_url,
    impact_score,
    extra,
    proposer_name,
    proposer_contact,
    proposer_url,
    proposer_relevant_experience,
    bb_proposal_id,
    bb_vote_options
) VALUES (
    1,
    CHALLENGE_ID,
    'Proposal Title',
    'Proposal Summary',
    'public',
    'public-key-1',
    100000,
    'https://proposal.com/1',
    'https://proposal.com/1/files',
    0.5,
    '{"solution": "Proposal solution"}',
    'Proposer Name',
    'Proposer Contact',
    'https://proposer.com',
    'Proposer Relevant Experience',
    'bb-proposal-id-1',
    'yes,no'
) RETURNING row_id;
```

```sql
INSERT INTO proposal_voteplan (
    proposal_id,
    voteplan_id,
    bb_proposal_index
) VALUES (
    PROPOSAL_ROW_ID,
    VOTEPLAN_ROW_ID,
    0
);
```
