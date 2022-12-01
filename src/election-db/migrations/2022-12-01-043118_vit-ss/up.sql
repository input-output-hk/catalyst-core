-- Catalyst Election Database - VIT-SS Compatibility 

-- This view maps the original VIT-SS funds table to the new election table.  
--   Do not use this VIEW for new queries, its ONLY for backward compatibility.
CREATE VIEW funds AS SELECT
    row_id AS id,
    name AS fund_name,
    description AS fund_goal,

    EXTRACT (EPOCH FROM registration_snapshot_time) AS registration_snapshot_time,
    NULL AS next_registration_snapshot_time, -- TODO - Get from next sequential election
    voting_power_threshold,
    EXTRACT (EPOCH FROM start_time) AS fund_start_time,
    EXTRACT (EPOCH FROM end_time) AS fund_end_time,
    NULL AS next_fund_start_time, -- TODO - Get from next sequential election
    EXTRACT (EPOCH FROM insight_sharing_start) AS insight_sharing_start,
    EXTRACT (EPOCH FROM proposal_submission_start) AS proposal_submission_start,
    EXTRACT (EPOCH FROM refine_proposals_start) AS refine_proposals_start,
    EXTRACT (EPOCH FROM finalize_proposals_start) AS finalize_proposals_start,
    EXTRACT (EPOCH FROM proposal_assessment_start) AS proposal_assessment_start,
    EXTRACT (EPOCH FROM assessment_qa_start) AS assessment_qa_start,
    EXTRACT (EPOCH FROM snapshot_start) AS snapshot_start,
    EXTRACT (EPOCH FROM voting_start) AS voting_start,
    EXTRACT (EPOCH FROM voting_end) AS voting_end,
    EXTRACT (EPOCH FROM tallying_end) AS tallying_end,

    extra->'url'->'results' AS results_url,
    extra->'url'->'survey' AS survey_url
FROM election;

COMMENT ON VIEW funds IS 
    'This view maps the original VIT-SS funds table to the new election table.  
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

/*
-- 

CREATE VIEW challenges AS SELECT
    row_id AS internal_id,
    id,
    election AS fund_id,
    rewards_total,
    proposers_rewards,
    extra->'url'->'challenge' AS challenge_url,
    extra->'highlights' AS highlights
FROM challenge;

COMMENT ON VIEW challenges IS 
    'This view maps the original VIT-SS challenges table to the new challenge table.  
Do not use this VIEW for new queries, its ONLY for backward compatibility.';


CREATE VIEW proposals AS SELECT 
    row_id AS id,
    id AS proposal_id,
    NULL AS proposal_category, --  TO DO, Get from a join with Election.
    title AS proposal_title,
    summary AS proposal_summary,
    public_key AS proposal_public_key,
    funds AS proposal_funds,
    url AS proposal_url,
    files_url AS proposal_files_url,
    proposer_name,
    proposer_contact,
    proposer_url,
    proposer_relevant_experience,
    bb_proposal_id AS chain_proposal_id,
    bb_vote_options AS chain_vote_options,
    challenge  AS challenge_id
FROM proposal;
    
COMMENT ON VIEW proposals IS 
    'This view maps the original VIT-SS proposals table to the new proposal table.  
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

-- Compatibility view for vit-ss
CREATE VIEW proposals_voteplans AS SELECT
    row_id AS id,
    CAST(proposal.id, VARCHAR) AS proposal_id,
    voteplan.id AS chain_voteplan_id,
    bb_proposal_index AS chain_proposal_index
FROM proposal_voteplan
    INNER JOIN proposal WHERE proposal_voteplan.proposal_id = proposal.row_id
    INNER JOIN voteplan WHERE proposal_voteplan.voteplan_id = voteplan.row_id;

COMMENT ON VIEW proposals_voteplans IS 
    'This view maps the original VIT-SS proposals table to the new proposal table.  
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

CREATE VIEW proposal_simple_challenge AS SELECT
    CAST(proposal.id AS VARCHAR) AS proposal_id,
    proposal.extra->'solution' AS proposal_solution  
FROM 
    proposal
    INNER JOIN challenge ON proposal.challenge = challenge.row_id
WHERE challenge.type = 'simple';

COMMENT ON VIEW proposal_simple_challenge IS 
    'This view maps the original VIT-SS proposal_simple_challenge table to the new proposal table.  
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

CREATE VIEW proposal_community_choice_challenge AS SELECT
    CAST(proposal.id AS VARCHAR) AS proposal_id,
    proposal.extra->'solution' AS proposal_solution  
    proposal.extra->'brief' AS proposal_brief,
    proposal.extra->'importance' AS proposal_importance,
    proposal.extra->'goal' AS proposal_goal,
    proposal.extra->'metrics' AS proposal_metrics
FROM 
    proposal
    INNER JOIN challenge ON proposal.challenge = challenge.row_id
WHERE challenge.type = 'community-choice';

COMMENT ON VIEW proposal_community_choice_challenge IS 
    'This view maps the original VIT-SS proposal_community_choice_challenge table to the new proposal table.  
Do not use this VIEW for new queries, its ONLY for backward compatibility.';

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

CREATE VIEW goals AS SELECT
    id,
    name AS goal_name,
    election_id AS fund_id,
FROM goal ORDER BY (election_id, idx);

-- api_tokens does not have a view because the old method of storing is insecure.
-- the token itself should be encrypted with a secret.
*/