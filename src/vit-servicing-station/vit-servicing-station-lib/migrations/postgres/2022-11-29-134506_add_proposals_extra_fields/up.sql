ALTER TABLE proposals ADD proposal_extra_fields TEXT;

-- Recreate view so it's updated with the added field
DROP VIEW full_proposals_info;
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
