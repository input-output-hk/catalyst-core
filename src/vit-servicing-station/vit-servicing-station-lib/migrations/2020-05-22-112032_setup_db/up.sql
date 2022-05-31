create table funds
(
    id INTEGER NOT NULL
        primary key autoincrement,
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
    id INTEGER NOT NULL
        primary key autoincrement,
    proposal_id VARCHAR NOT NULL,
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
    chain_proposal_id BLOB NOT NULL,
    chain_proposal_index BIGINT NOT NULL,
    chain_vote_options VARCHAR NOT NULL,
    chain_voteplan_id VARCHAR NOT NULL,
    challenge_id INTEGER NOT NULL
);

create table proposal_simple_challenge (
    proposal_id VARCHAR NOT NULL primary key,
    proposal_solution VARCHAR
);

create table proposal_community_choice_challenge (
    proposal_id VARCHAR NOT NULL primary key,
    proposal_brief VARCHAR,
    proposal_importance VARCHAR,
    proposal_goal VARCHAR,
    proposal_metrics VARCHAR
);

create table voteplans
(
    id INTEGER NOT NULL
        primary key autoincrement,
    chain_voteplan_id VARCHAR NOT NULL
        unique,
    chain_vote_start_time BIGINT NOT NULL,
    chain_vote_end_time BIGINT NOT NULL,
    chain_committee_end_time BIGINT NOT NULL,
    chain_voteplan_payload VARCHAR NOT NULL,
    chain_vote_encryption_key VARCHAR NOT NULL,
    fund_id INTEGER NOT NULL
);

create table api_tokens
(
    token BLOB NOT NULL UNIQUE PRIMARY KEY ,
    creation_time BIGINT NOT NULL,
    expire_time BIGINT NOT NULL
);

create table challenges
(
    internal_id INTEGER NOT NULL
        primary key autoincrement,
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
  id INTEGER NOT NULL primary key autoincrement,
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
    id INTEGER NOT NULL
        primary key autoincrement,
    goal_name VARCHAR NOT NULL,
    fund_id INTEGER NOT NULL,
    FOREIGN KEY(fund_id) REFERENCES funds(id)
);

CREATE VIEW full_proposals_info
AS
SELECT
    proposals.*,
    ifnull(reviews_count, 0) as reviews_count,
    proposal_simple_challenge.proposal_solution,
    proposal_community_choice_challenge.proposal_brief,
    proposal_community_choice_challenge.proposal_importance,
    proposal_community_choice_challenge.proposal_goal,
    proposal_community_choice_challenge.proposal_metrics,
    voteplans.chain_vote_start_time,
    voteplans.chain_vote_end_time,
    voteplans.chain_committee_end_time,
    voteplans.chain_voteplan_payload,
    voteplans.chain_vote_encryption_key,
    voteplans.fund_id,
    challenges.challenge_type
FROM
    proposals
        INNER JOIN voteplans ON proposals.chain_voteplan_id = voteplans.chain_voteplan_id
        INNER JOIN challenges on challenges.id = proposals.challenge_id
        LEFT JOIN proposal_simple_challenge
            on proposals.proposal_id = proposal_simple_challenge.proposal_id
            and challenges.challenge_type = 'simple'
        LEFT JOIN proposal_community_choice_challenge
            on proposals.proposal_id = proposal_community_choice_challenge.proposal_id
            and challenges.challenge_type = 'community-choice'
        LEFT JOIN (SELECT proposal_id as review_proposal_id, COUNT (DISTINCT assessor) as reviews_count FROM community_advisors_reviews GROUP BY proposal_id)
            on proposals.proposal_id = review_proposal_id;
