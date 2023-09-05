-- Catalyst Event Database - VIT-SS Compatibility

-- All Tables defined here will be dropped once Vit-SS backward compatibility is no longer needed.

-- This view maps the original VIT-SS funds table to the new event table.
--   Do not use this VIEW for new queries, its ONLY for backward compatibility.
CREATE VIEW funds AS SELECT
    this_fund.row_id AS id,
    this_fund.name AS fund_name,
    this_fund.description AS fund_goal,

    EXTRACT (EPOCH FROM this_fund.registration_snapshot_time)::BIGINT AS registration_snapshot_time,
    COALESCE(EXTRACT (EPOCH FROM next_fund.registration_snapshot_time), 0)::BIGINT AS next_registration_snapshot_time,

    this_fund.voting_power_threshold AS voting_power_threshold,

    EXTRACT (EPOCH FROM this_fund.start_time)::BIGINT AS fund_start_time,
    EXTRACT (EPOCH FROM this_fund.end_time)::BIGINT AS fund_end_time,
    COALESCE(EXTRACT (EPOCH FROM next_fund.start_time)::BIGINT, 0) AS next_fund_start_time,

    EXTRACT (EPOCH FROM this_fund.insight_sharing_start)::BIGINT AS insight_sharing_start,
    EXTRACT (EPOCH FROM this_fund.proposal_submission_start)::BIGINT AS proposal_submission_start,
    EXTRACT (EPOCH FROM this_fund.refine_proposals_start)::BIGINT AS refine_proposals_start,
    EXTRACT (EPOCH FROM this_fund.finalize_proposals_start)::BIGINT AS finalize_proposals_start,
    EXTRACT (EPOCH FROM this_fund.proposal_assessment_start)::BIGINT AS proposal_assessment_start,
    EXTRACT (EPOCH FROM this_fund.assessment_qa_start)::BIGINT AS assessment_qa_start,
    EXTRACT (EPOCH FROM this_fund.snapshot_start)::BIGINT AS snapshot_start,
    EXTRACT (EPOCH FROM this_fund.voting_start)::BIGINT AS voting_start,
    EXTRACT (EPOCH FROM this_fund.voting_end)::BIGINT AS voting_end,
    EXTRACT (EPOCH FROM this_fund.tallying_end)::BIGINT AS tallying_end,

    (this_fund.extra->'url'->'results') #>> '{}' AS results_url,
    (this_fund.extra->'url'->'survey') #>> '{}' AS survey_url
FROM event this_fund
LEFT JOIN event next_fund ON next_fund.row_id = this_fund.row_id + 1;

COMMENT ON VIEW funds IS
    '@omit
This view maps the original VIT-SS funds table to the new event table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - proposals table.

CREATE VIEW proposals AS SELECT
    proposal.row_id AS id,
    CAST(proposal.id AS VARCHAR) AS proposal_id,
    proposal.category AS proposal_category,
    proposal.title AS proposal_title,
    proposal.summary AS proposal_summary,
    proposal.public_key AS proposal_public_key,
    proposal.funds AS proposal_funds,
    proposal.url AS proposal_url,
    proposal.files_url AS proposal_files_url,
    proposal.impact_score  AS proposal_impact_score,

    proposal.proposer_name AS proposer_name,
    proposal.proposer_contact AS proposer_contact,
    proposal.proposer_url AS proposer_url,
    proposal.proposer_relevant_experience AS proposer_relevant_experience,

    proposal.bb_proposal_id AS chain_proposal_id,
    proposal.bb_vote_options AS chain_vote_options,
    objective.id  AS challenge_id,

    proposal.extra #>> '{}' AS extra
FROM proposal
INNER JOIN objective ON objective.row_id = proposal.objective;

COMMENT ON VIEW proposals IS
    '@omit
This view maps the original VIT-SS proposals table to the new proposal table.
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
    '@omit
This view maps the original VIT-SS proposals table to the new proposal table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - proposal_simple_challenge table.

CREATE VIEW proposal_simple_challenge AS SELECT
    CAST(proposal.id AS VARCHAR) AS proposal_id,
    (proposal.extra->'solution') #>> '{}' AS proposal_solution
FROM
    proposal
    INNER JOIN objective ON proposal.objective = objective.row_id
WHERE objective.category = 'catalyst-simple';

COMMENT ON VIEW proposal_simple_challenge IS
    '@omit
This view maps the original VIT-SS proposal_simple_challenge table to the new proposal table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - proposal_community_choice_challenge.

CREATE VIEW proposal_community_choice_challenge AS SELECT
    CAST(proposal.id AS VARCHAR) AS proposal_id,
    (proposal.extra->'solution') #>> '{}' AS proposal_solution,
    (proposal.extra->'brief') #>> '{}' AS proposal_brief,
    (proposal.extra->'importance') #>> '{}' AS proposal_importance,
    (proposal.extra->'goal') #>> '{}' AS proposal_goal,
    (proposal.extra->'metrics') #>> '{}' AS proposal_metrics
FROM
    proposal
    INNER JOIN objective ON proposal.objective = objective.row_id
WHERE objective.category = 'catalyst-community-choice';


COMMENT ON VIEW proposal_community_choice_challenge IS
    '@omit
This view maps the original VIT-SS proposal_community_choice_challenge table to the new proposal table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - voteplans table.

CREATE VIEW voteplans AS SELECT
    voteplan.row_id AS id,
    voteplan.id AS chain_voteplan_id,
    EXTRACT (EPOCH FROM event.voting_start)::BIGINT AS chain_vote_start_time,
    EXTRACT (EPOCH FROM event.voting_end)::BIGINT AS chain_vote_end_time,
    EXTRACT (EPOCH FROM event.tallying_end)::BIGINT AS chain_committee_end_time,
    voteplan.category AS chain_voteplan_payload,
    voteplan.encryption_key AS chain_vote_encryption_key,
    event.row_id AS fund_id,
    voteplan.token_id AS token_identifier
FROM voteplan
    INNER JOIN objective ON voteplan.objective_id = objective.row_id
    INNER JOIN event ON objective.event = event.row_id;

COMMENT ON VIEW voteplans IS
    '@omit
This view maps the original VIT-SS voteplans table to the new voteplan table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - api_tokens table.

CREATE VIEW api_tokens AS SELECT
    DECODE(config.id2, 'base64')::BYTEA AS token,
    (config.value->'created')::BIGINT AS creation_time,
    (config.value->'expires')::BIGINT AS expire_time
FROM config
    WHERE config.id = 'api_token';

COMMENT ON VIEW api_tokens IS
    '@omit
This view maps the original VIT-SS api_tokens table to the new config table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.
This table uses unencrypted values, so is not compatible with api tokens that are
encrypted.  It should be obsoleted at the earliest opportunity.';

-- VIT-SS Compatibility View - challenges table.

CREATE VIEW challenges AS SELECT
    objective.row_id AS internal_id,
    objective.id AS id,
    objective.category AS challenge_type,
    objective.title AS title,
    objective.description AS description,
    objective.rewards_total AS rewards_total,
    objective.proposers_rewards AS proposers_rewards,
    objective.event AS fund_id,
    (objective.extra->'url'->'objective') #>> '{}' AS challenge_url,
    (objective.extra->'highlights') #>> '{}' AS highlights
FROM objective;

COMMENT ON VIEW challenges IS
    '@omit
This view maps the original VIT-SS challenges table to the new challenge table.
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
FROM proposal_review;

COMMENT ON VIEW community_advisors_reviews IS
    '@omit
This view maps the original VIT-SS community_advisors_reviews table to the new proposal_review table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - goals.

CREATE VIEW goals AS SELECT
    id,
    name AS goal_name,
    event_id AS fund_id
FROM goal ORDER BY (event_id, idx);

COMMENT ON VIEW goals IS
    '@omit
This view maps the original VIT-SS goals table to the new goal table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Compatibility View - groups.

CREATE VIEW groups AS SELECT
     objective.event AS fund_id,
     voteplan.token_id AS token_identifier,
     voting_group.name AS group_id
 FROM voting_group
    INNER JOIN voteplan ON voteplan.group_id = voting_group.name
    INNER JOIN objective ON voteplan.objective_id = objective.row_id;

COMMENT ON VIEW groups IS
    '@omit
This view maps the original VIT-SS groups table to the new voting_groups table.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- VIT-SS Full Proposal View

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
    voteplan.group_id
FROM proposals p
INNER JOIN proposals_voteplans pvp ON p.id::VARCHAR = pvp.proposal_id
INNER JOIN voteplans vp ON pvp.chain_voteplan_id = vp.chain_voteplan_id
INNER JOIN challenges ch ON ch.id = p.challenge_id
INNER JOIN voteplan ON voteplan.id = vp.chain_voteplan_id
LEFT JOIN proposal_simple_challenge psc
    ON p.proposal_id = psc.proposal_id
    AND ch.challenge_type = 'catalyst-simple'
LEFT JOIN proposal_community_choice_challenge pccc
    ON p.proposal_id = pccc.proposal_id
    AND ch.challenge_type = 'catalyst-community-choice'
LEFT JOIN (
        SELECT
            proposal_id::VARCHAR AS review_proposal_id,
            COUNT (DISTINCT assessor)::INTEGER AS reviews_count
        FROM community_advisors_reviews
        GROUP BY proposal_id
    ) rev ON p.proposal_id = rev.review_proposal_id;

COMMENT ON VIEW full_proposals_info IS
    '@omit
This view maps the original VIT-SS full proposals view.
Do not use this VIEW for new queries, its ONLY for backward compatibility.';
