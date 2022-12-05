-- This file should undo anything in `up.sql`
DROP VIEW IF EXISTS
    schema_version;
    
DROP TABLE IF EXISTS 
    config,
    voteplan_types,
    voting_group,
    voteplan,
    proposal_voteplan,
    community_advisors_review,
    goal,
    voter_registration_type,
    voter_registration,
    stake_address_balance,
    voting_power,
    election,
    challenge_category,
    currency,
    vote_options,
    challenge,
    proposal;

