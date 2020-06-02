-- Your SQL goes here

CREATE TABLE proposals (
    proposal_id VARCHAR NOT NULL UNIQUE PRIMARY KEY,
    proposal_category VARCHAR NOT NULL,
    proposal_title VARCHAR NOT NULL,
    proposal_summary VARCHAR NOT NULL,
    proposal_problem VARCHAR NOT NULL,
    proposal_solution VARCHAR NOT NULL,
    proposal_funds BIGINT NOT NULL,
    proposal_url VARCHAR NOT NULL,
    proposal_files_url VARCHAR NOT NULL,
    proposer_name VARCHAR NOT NULL,
    proposer_contact VARCHAR NOT NULL,
    proposer_url VARCHAR NOT NULL,
    chain_proposal_id VARCHAR NOT NULL,
    chain_voteplan_id VARCHAR NOT NULL,
    chain_proposal_index BIGINT NOT NULL,
    chain_vote_start_time BIGINT NOT NULL,
    chain_vote_end_time BIGINT NOT NULL,
    chain_committee_end_time BIGINT NOT NULL,
    chain_vote_options VARCHAR NOT NULL
);

CREATE TABLE funds (
    fund_name VARCHAR NOT NULL UNIQUE PRIMARY KEY,
    fund_goal VARCHAR NOT NULL,
    voting_power_info VARCHAR NOT NULL,
    rewards_info VARCHAR NOT NULL,
    fund_start_time VARCHAR NOT NULL,
    fund_end_time VARCHAR NOT NULL,
    next_fund_start_time VARCHAR NOT NULL
);

CREATE TABLE chain_voteplan (
    vote_plan_id VARCHAR NOT NULL UNIQUE PRIMARY KEY,
    chain_vote_plan_id VARCHAR NOT NULL,
    chain_vote_starttime VARCHAR NOT NULL,
    chain_vote_endtime VARCHAR NOT NULL,
    chain_committee_endtime VARCHAR NOT NULL
);

CREATE TABLE fund_voteplans (
    fund_name VARCHAR NOT NULL,
    vote_plan_id VARCHAR NOT NULL,
    PRIMARY KEY (fund_name, vote_plan_id)
)