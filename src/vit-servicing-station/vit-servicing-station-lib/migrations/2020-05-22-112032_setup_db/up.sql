create table funds
(
    id SERIAL PRIMARY KEY,
    fund_name VARCHAR NOT NULL,
    fund_goal VARCHAR NOT NULL,
    registration_snapshot_time BIGINT NOT NULL,
    next_registration_snapshot_time BIGINT NOT NULL,
    voting_power_threshold BIGINT NOT NULL,
    fund_start_time BIGINT NOT NULL,
    fund_end_time BIGINT NOT NULL,
    next_fund_start_time BIGINT NOT NULL,
    insight_sharing_start BIGINT NOT NULL,
    proposal_submission_start BIGINT NOT NULL,
    refine_proposals_start BIGINT NOT NULL,
    finalize_proposals_start BIGINT NOT NULL,
    proposal_assessment_start BIGINT NOT NULL,
    assessment_qa_start BIGINT NOT NULL,
    snapshot_start BIGINT NOT NULL,
    voting_start BIGINT NOT NULL,
    voting_end BIGINT NOT NULL,
    tallying_end BIGINT NOT NULL,
    results_url VARCHAR NOT NULL,
    survey_url VARCHAR NOT NULL
);

create table proposals
(
    id SERIAL PRIMARY KEY,
    proposal_id VARCHAR NOT NULL UNIQUE,
    proposal_category VARCHAR NOT NULL,
    proposal_title VARCHAR NOT NULL,
    proposal_summary VARCHAR NOT NULL,
    proposal_public_key VARCHAR NOT NULL,
    proposal_funds BIGINT NOT NULL,
    proposal_url VARCHAR NOT NULL,
    proposal_files_url VARCHAR NOT NULL,
    proposal_impact_score BIGINT NOT NULL,
    proposer_name VARCHAR NOT NULL,
    proposer_contact VARCHAR NOT NULL,
    proposer_url VARCHAR NOT NULL,
    proposer_relevant_experience VARCHAR NOT NULL,
    chain_proposal_id BYTEA NOT NULL,
    chain_vote_options VARCHAR NOT NULL,
    challenge_id INTEGER NOT NULL
);

create table proposals_voteplans (
    id SERIAL,
    proposal_id VARCHAR NOT NULL,
    chain_voteplan_id VARCHAR NOT NULL,
    chain_proposal_index BIGINT NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (proposal_id) REFERENCES proposals (proposal_id)
);

create table proposal_simple_challenge (
    proposal_id VARCHAR NOT NULL PRIMARY KEY,
    proposal_solution VARCHAR
);

create table proposal_community_choice_challenge (
    proposal_id VARCHAR NOT NULL PRIMARY KEY,
    proposal_brief VARCHAR,
    proposal_importance VARCHAR,
    proposal_goal VARCHAR,
    proposal_metrics VARCHAR
);

create table voteplans
(
    id SERIAL PRIMARY KEY,
    chain_voteplan_id VARCHAR NOT NULL
        unique,
    chain_vote_start_time BIGINT NOT NULL,
    chain_vote_end_time BIGINT NOT NULL,
    chain_committee_end_time BIGINT NOT NULL,
    chain_voteplan_payload VARCHAR NOT NULL,
    chain_vote_encryption_key VARCHAR NOT NULL,
    fund_id INTEGER NOT NULL,
    token_identifier VARCHAR NOT NULL
);

create table api_tokens
(
    token BYTEA PRIMARY KEY,
    creation_time BIGINT NOT NULL,
    expire_time BIGINT NOT NULL
);

create table challenges
(
    internal_id SERIAL PRIMARY KEY,
    id INTEGER NOT NULL UNIQUE,
    challenge_type VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    rewards_total BIGINT NOT NULL,
    proposers_rewards BIGINT NOT NULL,
    fund_id INTEGER NOT NULL,
    challenge_url VARCHAR NOT NULL,
    highlights VARCHAR
);

create table community_advisors_reviews (
  id SERIAL PRIMARY KEY,
  proposal_id INTEGER NOT NULL,
  assessor VARCHAR NOT NULL,
  impact_alignment_rating_given INTEGER NOT NULL,
  impact_alignment_note VARCHAR NOT NULL,
  feasibility_rating_given INTEGER NOT NULL,
  feasibility_note VARCHAR NOT NULL,
  auditability_rating_given INTEGER NOT NULL,
  auditability_note VARCHAR NOT NULL,
  ranking INTEGER NOT NULL
);

create table goals
(
    id SERIAL PRIMARY KEY,
    goal_name VARCHAR NOT NULL,
    fund_id INTEGER NOT NULL,
    FOREIGN KEY(fund_id) REFERENCES funds(id)
);

create table groups (
    fund_id INTEGER NOT NULL,
    token_identifier VARCHAR,
    group_id VARCHAR NOT NULL,
    PRIMARY KEY(token_identifier, fund_id)
);


create table votes (
    fragment_id TEXT,
    caster TEXT NOT NULL,
    proposal INTEGER NOT NULL,
    voteplan_id TEXT NOT NULL,
    time REAL NOT NULL,
    choice SMALLINT,
    raw_fragment TEXT NOT NULL,
    PRIMARY KEY(fragment_id)
);

create table snapshots (
    tag TEXT PRIMARY KEY,
    last_updated BIGINT NOT NULL
);

create table voters (
    voting_key TEXT NOT NULL,
    voting_power BIGINT NOT NULL,
    voting_group TEXT NOT NULL,
    snapshot_tag TEXT NOT NULL,
    PRIMARY KEY(voting_key, voting_group, snapshot_tag),
    FOREIGN KEY(snapshot_tag) REFERENCES snapshots(tag) ON DELETE CASCADE
);

create table contributions (
    stake_public_key TEXT NOT NULL,
    reward_address TEXT NOT NULL,
    value BIGINT NOT NULL,
    voting_key TEXT NOT NULL,
    voting_group TEXT NOT NULL,
    snapshot_tag TEXT NOT NULL,
    PRIMARY KEY(stake_public_key, voting_key, voting_group, snapshot_tag),
    FOREIGN KEY(snapshot_tag) REFERENCES snapshots(tag) ON DELETE CASCADE
);

CREATE VIEW full_proposals_info AS
SELECT
    p.*,
    COALESCE(reviews_count, 0) as reviews_count,
    vp.chain_vote_start_time,
    vp.chain_vote_end_time,
    vp.chain_committee_end_time,
    vp.chain_voteplan_payload,
    vp.chain_vote_encryption_key,
    vp.fund_id,
    ch.challenge_type,
    psc.proposal_solution,
    pccc.proposal_brief,
    pccc.proposal_importance,
    pccc.proposal_goal,
    pccc.proposal_metrics,
    pvp.chain_proposal_index,
    pvp.chain_voteplan_id,
    gr.group_id
FROM proposals p
INNER JOIN proposals_voteplans pvp ON p.proposal_id = pvp.proposal_id
INNER JOIN voteplans vp ON pvp.chain_voteplan_id = vp.chain_voteplan_id
INNER JOIN challenges ch ON ch.id = p.challenge_id
INNER JOIN groups gr ON vp.token_identifier = gr.token_identifier
LEFT JOIN proposal_simple_challenge psc
    ON p.proposal_id = psc.proposal_id
    AND ch.challenge_type = 'simple'
LEFT JOIN proposal_community_choice_challenge pccc
    ON p.proposal_id = pccc.proposal_id
    AND ch.challenge_type = 'community-choice'
LEFT JOIN (
        SELECT
            proposal_id::VARCHAR AS review_proposal_id,
            COUNT (DISTINCT assessor)::INTEGER AS reviews_count
        FROM community_advisors_reviews
        GROUP BY proposal_id
    ) rev ON p.proposal_id = rev.review_proposal_id;
