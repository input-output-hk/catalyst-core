-- Catalyst Election Database - VIT-SS Compatibility

-- This view maps the original VIT-SS funds table to the new election table.
--   Do not use this VIEW for new queries, its ONLY for backward compatibility.
CREATE VIEW funds AS SELECT
    this_fund.row_id AS id,
    this_fund.name AS fund_name,
    this_fund.description AS fund_goal,

    EXTRACT (EPOCH FROM this_fund.registration_snapshot_time) AS registration_snapshot_time,
    EXTRACT (EPOCH FROM next_fund.registration_snapshot_time) AS next_registration_snapshot_time,

    this_fund.voting_power_threshold AS voting_power_threshold,

    EXTRACT (EPOCH FROM this_fund.start_time) AS fund_start_time,
    EXTRACT (EPOCH FROM this_fund.end_time) AS fund_end_time,
    EXTRACT (EPOCH FROM next_fund.start_time) AS next_fund_start_time,

    EXTRACT (EPOCH FROM this_fund.insight_sharing_start) AS insight_sharing_start,
    EXTRACT (EPOCH FROM this_fund.proposal_submission_start) AS proposal_submission_start,
    EXTRACT (EPOCH FROM this_fund.refine_proposals_start) AS refine_proposals_start,
    EXTRACT (EPOCH FROM this_fund.finalize_proposals_start) AS finalize_proposals_start,
    EXTRACT (EPOCH FROM this_fund.proposal_assessment_start) AS proposal_assessment_start,
    EXTRACT (EPOCH FROM this_fund.assessment_qa_start) AS assessment_qa_start,
    EXTRACT (EPOCH FROM this_fund.snapshot_start) AS snapshot_start,
    EXTRACT (EPOCH FROM this_fund.voting_start) AS voting_start,
    EXTRACT (EPOCH FROM this_fund.voting_end) AS voting_end,
    EXTRACT (EPOCH FROM this_fund.tallying_end) AS tallying_end,

    this_fund.extra->'url'->'results' AS results_url,
    this_fund.extra->'url'->'survey' AS survey_url
FROM election this_fund
INNER JOIN election next_fund ON next_fund.row_id == this_fund.row_id + 1;

COMMENT ON VIEW funds IS
    'This view maps the original VIT-SS funds table to the new election table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - proposals table.

CREATE VIEW proposals AS SELECT
    proposal.row_id AS id,
    CAST(proposal.id AS VARCHAR) AS proposal_id,
    challenge.category AS proposal_category,
    proposal.title AS proposal_title,
    proposal.summary AS proposal_summary,
    proposal.public_key AS proposal_public_key,
    proposal.funds AS proposal_funds,
    proposal.url AS proposal_url,
    proposal.files_url AS proposal_files_url,
    proposal.impact_score AS impace_score,

    proposal.proposer_name AS proposer_name,
    proposal.proposer_contact AS proposer_contact,
    proposal.proposer_url AS proposer_url,
    proposal.proposer_relevant_experience AS proposer_relevant_experience,

    proposal.bb_proposal_id AS chain_proposal_id,
    proposal.bb_vote_options AS chain_vote_options,
    proposal.challenge  AS challenge_id
FROM proposal
INNER JOIN challenge ON challenge.row_id = proposal.challenge;

COMMENT ON VIEW proposals IS
    'This view maps the original VIT-SS proposals table to the new proposal table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - proposals_voteplans table.
CREATE VIEW proposals_voteplans AS SELECT
    proposal_voteplan.row_id AS id,
    CAST(proposal.id AS VARCHAR) AS proposal_id,
    voteplan.id AS chain_voteplan_id,
    bb_proposal_index AS chain_proposal_index
FROM proposal_voteplan
    INNER JOIN proposal ON proposal_voteplan.proposal_id = proposal.row_id
    INNER JOIN voteplan ON proposal_voteplan.voteplan_id = voteplan.row_id;

COMMENT ON VIEW proposals_voteplans IS
    'This view maps the original VIT-SS proposals table to the new proposal table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - proposal_simple_challenge table.

CREATE VIEW proposal_simple_challenge AS SELECT
    CAST(proposal.id AS VARCHAR) AS proposal_id,
    proposal.extra->'solution' AS proposal_solution
FROM
    proposal
    INNER JOIN challenge ON proposal.challenge = challenge.row_id
WHERE challenge.category = 'simple';

COMMENT ON VIEW proposal_simple_challenge IS
    'This view maps the original VIT-SS proposal_simple_challenge table to the new proposal table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - proposal_community_choice_challenge.

CREATE VIEW proposal_community_choice_challenge AS SELECT
    CAST(proposal.id AS VARCHAR) AS proposal_id,
    proposal.extra->'solution' AS proposal_solution,
    proposal.extra->'brief' AS proposal_brief,
    proposal.extra->'importance' AS proposal_importance,
    proposal.extra->'goal' AS proposal_goal,
    proposal.extra->'metrics' AS proposal_metrics
FROM
    proposal
    INNER JOIN challenge ON proposal.challenge = challenge.row_id
WHERE challenge.category = 'community-choice';

COMMENT ON VIEW proposal_community_choice_challenge IS
    'This view maps the original VIT-SS proposal_community_choice_challenge table to the new proposal table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - voteplans table.

CREATE VIEW voteplans AS SELECT
    voteplan.row_id AS id,
    voteplan.id AS chain_voteplan_id,
    EXTRACT (EPOCH FROM election.voting_start) AS chain_vote_start_time,
    EXTRACT (EPOCH FROM election.voting_end) AS chain_vote_end_time,
    EXTRACT (EPOCH FROM election.tallying_end) AS chain_committee_end_time,
    voteplan.category AS chain_voteplan_payload,
    voteplan.encryption_key AS chain_vote_encryption_key,
    election.row_id AS fund_id,
    voting_group.token_id AS token_identifier
FROM voteplan
    INNER JOIN election ON voteplan.election_id = election.row_id
    INNER JOIN voting_group ON voteplan.group_id = voting_group.row_id;

COMMENT ON VIEW voteplans IS
    'This view maps the original VIT-SS voteplans table to the new voteplan table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - api_tokens table.

CREATE VIEW api_tokens AS SELECT
    DECODE(config.id2, 'base64') AS token,
    config.value->'created' AS creation_time,
    config.value->'expires' AS expire_time
FROM config
    WHERE config.id = 'api_token';

COMMENT ON VIEW api_tokens IS
    'This view maps the original VIT-SS api_tokens table to the new config table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.
This table uses unencrypted values, so is not compatible with api tokens that are
encrypted.  It should be obsoleted at the earliest opportunity.';

-- VIT-SS Compatibility View - challenges table.

CREATE VIEW challenges AS SELECT
    challenge.row_id AS internal_id,
    challenge.id AS id,
    challenge.category AS challenge_type,
    challenge.title AS title,
    challenge.description AS description,
    challenge.rewards_total AS rewards_total,
    challenge.proposers_rewards AS proposers_rewards,
    challenge.election AS fund_id,
    challenge.extra->'url'->'challenge' AS challenge_url,
    challenge.extra->'highlights' AS highlights
FROM challenge;

COMMENT ON VIEW challenges IS
    'This view maps the original VIT-SS challenges table to the new challenge table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - community_advisors_reviews table.

CREATE VIEW community_advisors_reviews AS SELECT
    row_id AS id,
    proposal_id,
    assessor,
    impact_alignment_rating_given,
    impact_alignment_note,
    feasibility_rating_given,
    feasibility_note,
    auditability_rating_given,
    auditability_note,
    ranking
FROM community_advisors_review;

COMMENT ON VIEW community_advisors_reviews IS
    'This view maps the original VIT-SS community_advisors_reviews table to the new community_advisors_review table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - goals.

CREATE VIEW goals AS SELECT
    id,
    name AS goal_name,
    election_id AS fund_id
FROM goal ORDER BY (election_id, idx);

COMMENT ON VIEW goals IS
    'This view maps the original VIT-SS goals table to the new goal table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - groups.

CREATE VIEW groups AS SELECT
    election_id AS fund_id,
    token_id AS token_identifier,
    group_id AS group_id
FROM voting_group;

COMMENT ON VIEW groups IS
    'This view maps the original VIT-SS groups table to the new voting_groups table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- These tables are not updated to the new schema and still need analysis.
-- They are a copy of the latest vit-ss database table definitions.

-- votes

CREATE TABLE votes (
    fragment_id TEXT PRIMARY KEY,
    caster TEXT NOT NULL,
    proposal INTEGER NOT NULL,
    voteplan_id TEXT NOT NULL,
    time REAL NOT NULL,
    choice SMALLINT,
    raw_fragment TEXT NOT NULL
);

-- TODO, Shouldn't this be related to the other tables?

COMMENT ON TABLE votes IS 'All Votes. VIT-SS Compatibility Table, to be replaced when analysis completed.';
COMMENT ON COLUMN votes.fragment_id is 'TODO';
COMMENT ON COLUMN votes.caster is 'TODO';
COMMENT ON COLUMN votes.proposal is 'TODO';
COMMENT ON COLUMN votes.voteplan_id is 'TODO';
COMMENT ON COLUMN votes.time is 'TODO';
COMMENT ON COLUMN votes.choice is 'TODO';
COMMENT ON COLUMN votes.raw_fragment is 'TODO';

-- snapshots

CREATE TABLE snapshots (
    tag TEXT PRIMARY KEY,
    last_updated TIMESTAMP NOT NULL
);

COMMENT ON TABLE snapshots IS 'Something to do with snapshots. VIT-SS Compatibility Table, to be replaced when analysis completed.';
COMMENT ON COLUMN snapshots.last_updated is 'TODO';

-- voters

CREATE TABLE voters (
    voting_key TEXT NOT NULL,
    voting_power BIGINT NOT NULL,
    voting_group TEXT NOT NULL,
    snapshot_tag TEXT NOT NULL,

    PRIMARY KEY(voting_key, voting_group, snapshot_tag),
    FOREIGN KEY(snapshot_tag) REFERENCES snapshots(tag) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_voter_id on voters (voting_key, voting_group, snapshot_tag);

COMMENT ON TABLE voters IS 'Voters table. VIT-SS Compatibility Table, to be replaced when analysis completed.';
COMMENT ON COLUMN voters.voting_key is 'TODO';
COMMENT ON COLUMN voters.voting_power is 'TODO';
COMMENT ON COLUMN voters.voting_group is 'TODO';
COMMENT ON COLUMN voters.snapshot_tag is 'TODO';

-- contributions

CREATE TABLE contributions (
    row_id SERIAL PRIMARY KEY,

    stake_public_key TEXT NOT NULL,
    voting_key TEXT NOT NULL,
    voting_group TEXT NOT NULL,
    snapshot_tag TEXT NOT NULL,

    reward_address TEXT NOT NULL,
    value BIGINT NOT NULL,

    FOREIGN KEY(snapshot_tag) REFERENCES snapshots(tag) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_contribution_id ON contributions (stake_public_key, voting_key, voting_group, snapshot_tag);

COMMENT ON TABLE contributions IS 'TODO';
COMMENT ON COLUMN contributions.row_id is 'Synthetic Unique Row Key';
COMMENT ON COLUMN contributions.stake_public_key IS 'TODO';
COMMENT ON COLUMN contributions.voting_key IS 'TODO';
COMMENT ON COLUMN contributions.voting_group IS 'TODO';
COMMENT ON COLUMN contributions.snapshot_tag IS 'TODO';
COMMENT ON COLUMN contributions.reward_address IS 'TODO';
COMMENT ON COLUMN contributions.value IS 'TODO';

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
