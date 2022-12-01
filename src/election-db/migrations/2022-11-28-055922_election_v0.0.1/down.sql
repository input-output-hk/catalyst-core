-- This file should undo anything in `up.sql`

DROP TABLE IF EXISTS config;
DROP TABLE IF EXISTS proposals_voteplans;
DROP TABLE IF EXISTS proposal_simple_challenge;
DROP TABLE IF EXISTS proposal_community_choice_challenge;
DROP TABLE IF EXISTS community_advisors_reviews;
DROP TABLE IF EXISTS goals;
DROP TABLE IF EXISTS groups;
DROP TABLE IF EXISTS votes;
DROP TABLE IF EXISTS voters;
DROP TABLE IF EXISTS contributions;
DROP TABLE IF EXISTS snapshots;
DROP TABLE IF EXISTS voteplans;

DROP TABLE IF EXISTS proposal;
DROP TABLE IF EXISTS challenge;
DROP TABLE IF EXISTS election;
DROP TABLE IF EXISTS challenge_type;
DROP TABLE IF EXISTS currency;
