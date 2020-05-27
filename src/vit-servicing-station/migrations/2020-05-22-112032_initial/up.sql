-- Your SQL goes here

CREATE TABLE proposals (
    id INTEGER PRIMARY KEY NOT NULL,
    proposal_category VARCHAR NOT NULL,
    proposal_id VARCHAR NOT NULL,
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
)