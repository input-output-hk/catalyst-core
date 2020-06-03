create table funds
(
    id INTEGER not null
        primary key autoincrement,
    fund_name VARCHAR not null,
    fund_goal VARCHAR not null,
    voting_power_info VARCHAR not null,
    rewards_info VARCHAR not null,
    fund_start_time VARCHAR not null,
    fund_end_time VARCHAR not null,
    next_fund_start_time VARCHAR not null
);

create table proposals
(
    id INTEGER not null
        primary key autoincrement,
    proposal_id VARCHAR not null,
    proposal_category VARCHAR not null,
    proposal_title VARCHAR not null,
    proposal_summary VARCHAR not null,
    proposal_problem VARCHAR not null,
    proposal_solution VARCHAR not null,
    proposal_public_key VARCHAR not null,
    proposal_funds BIGINT not null,
    proposal_url VARCHAR not null,
    proposal_files_url VARCHAR not null,
    proposer_name VARCHAR not null,
    proposer_contact VARCHAR not null,
    proposer_url VARCHAR not null,
    chain_proposal_id BLOB not null,
    chain_proposal_index BIGINT not null,
    chain_vote_options VARCHAR not null,
    chain_voteplan_id VARCHAR not null
);

create table voteplans
(
    id INTEGER not null
        primary key autoincrement,
    chain_voteplan_id VARCHAR not null
        unique,
    chain_vote_start_time VARCHAR not null,
    chain_vote_end_time VARCHAR not null,
    chain_committee_end_time VARCHAR not null,
    chain_voteplan_payload VARCHAR not null,
    fund_id INTEGER not null
);

CREATE VIEW full_proposals_info
AS
SELECT
    proposals.*,
    voteplans.chain_voteplan_payload,
    voteplans.chain_vote_start_time,
    voteplans.chain_vote_end_time,
    voteplans.chain_committee_end_time,
    voteplans.fund_id
FROM
    proposals
        INNER JOIN voteplans ON proposals.chain_voteplan_id = voteplans.chain_voteplan_id

