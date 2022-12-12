-- This file should undo anything in `up.sql`

-- DROP the VIT-SS Backwards compatibility views.
DROP VIEW IF EXISTS 
    funds,
    proposals,
    proposals_voteplans,
    proposal_simple_challenge,
    proposal_community_choice_challenge,
    voteplans,
    api_tokens,
    challenges,
    community_advisors_reviews,
    goals,
    groups,
    full_proposals_info;

-- DROP VIT_SS compatibility tables
DROP TABLE IF EXISTS
    votes,
    snapshots,
    voters,
    contributions;
