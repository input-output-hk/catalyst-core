-- This file should undo anything in `up.sql`

-- DROP the VIT-SS Backwards compatibility views.
DROP VIEW IF EXISTS funds;
DROP VIEW IF EXISTS challenges;
DROP VIEW IF EXISTS proposals;
DROP VIEW IF EXISTS proposals_voteplans;
DROP VIEW IF EXISTS proposal_simple_challenge;
DROP VIEW IF EXISTS proposal_community_choice_challenge;
