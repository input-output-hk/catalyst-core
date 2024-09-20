-- F13
INSERT INTO event (
    row_id,
    name,
    description,
    registration_snapshot_time,
    snapshot_start,
    voting_power_threshold,
    max_voting_power_pct,
    review_rewards,
    start_time,
    end_time,
    insight_sharing_start,
    proposal_submission_start,
    refine_proposals_start,
    finalize_proposals_start,
    proposal_assessment_start,
    assessment_qa_start,
    voting_start,
    voting_end,
    tallying_end,
    block0,
    block0_hash,
    committee_size,
    committee_threshold,
    extra,
    cast_to
) VALUES (
    13,
    'Fund 13',
    'Catalyst Testnet - Fund 13',
    '2024-10-12 11:00:00',  -- Registration Snapshot Time
    '2024-10-12 11:15:00',  -- Snapshot Start.
    2500000,              -- Voting Power Threshold
    100,                      -- Max Voting Power PCT
    NULL,                   -- Review Rewards
    '2024-09-20 10:00:00',  -- Start Time
    '2024-12-20 10:00:00',  -- End Time
    '2024-09-20 11:00:00',  -- Insight Sharing Start
    '2024-09-20 11:00:00',  -- Proposal Submission Start
    '2024-09-20 11:00:00',  -- Refine Proposals Start
    '2024-09-20 11:00:00',  -- Finalize Proposals Start
    '2024-09-20 11:00:00',  -- Proposal Assessment Start
    '2024-09-20 11:00:00',  -- Assessment QA Start
    '2024-10-13 11:00:00',  -- Voting Starts
    '2024-10-13 20:00:00',  -- Voting Ends
    '2024-12-20 10:00:00',  -- Tallying Ends
    NULL,                   -- Block 0 Data
    NULL,                   -- Block 0 Hash
    1,                      -- Committee Size
    1,                      -- Committee Threshold
    NULL,                   -- Extra
    NULL                    -- Cast to
) ON CONFLICT (row_id) DO UPDATE
SET name = EXCLUDED.name,
    description = EXCLUDED.description,
    registration_snapshot_time = EXCLUDED.registration_snapshot_time,
    snapshot_start = EXCLUDED.snapshot_start,
    voting_power_threshold = EXCLUDED.voting_power_threshold,
    max_voting_power_pct = EXCLUDED.max_voting_power_pct,
    review_rewards = EXCLUDED.review_rewards,
    start_time = EXCLUDED.start_time,
    end_time = EXCLUDED.end_time,
    insight_sharing_start = EXCLUDED.insight_sharing_start,
    proposal_submission_start = EXCLUDED.proposal_submission_start,
    refine_proposals_start = EXCLUDED.refine_proposals_start,
    finalize_proposals_start = EXCLUDED.finalize_proposals_start,
    proposal_assessment_start = EXCLUDED.proposal_assessment_start,
    assessment_qa_start = EXCLUDED.assessment_qa_start,
    voting_start = EXCLUDED.voting_start,
    voting_end = EXCLUDED.voting_end,
    tallying_end = EXCLUDED.tallying_end,
    block0 = EXCLUDED.block0,
    block0_hash = EXCLUDED.block0_hash,
    committee_size = EXCLUDED.committee_size,
    committee_threshold = EXCLUDED.committee_threshold,
    extra = EXCLUDED.extra,
    cast_to = EXCLUDED.cast_to;
