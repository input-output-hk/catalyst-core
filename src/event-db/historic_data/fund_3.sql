-- Data from Catalyst Fund 2 - First Public Test

-- Data Sources see:
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2892759058/Catalyst+Fund+Cycle+Releases
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2088075265/Fund-3
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2155282660/Fund+3+archive
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2155282739/F3+General+governance+parameters
-- https://github.com/input-output-hk/catalyst-resources/blob/master/snapshots/snapshots.json

-- Purge all Fund 1 data before re-inserting it.
DELETE FROM event WHERE row_id = 3;

-- Load the raw Block0 Binary from the file.
\set block0path 'historic_data/fund_3/block0.bin'
\set block0contents `base64 :block0path`

-- Create the Event record for Fund 0

INSERT INTO event
(row_id, name, description,
 start_time,
 end_time,
 registration_snapshot_time,
 snapshot_start,
 voting_power_threshold,
 max_voting_power_pct,
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
 committee_threshold)
VALUES

(3, 'Catalyst Fund 3', '',
 '2021-01-06 21:00:00', -- Start Time - Date/Time accurate.
 '2021-04-02 19:00:00', -- End Time   - Date/Time accurate.
 '2021-03-05 19:00:53', -- Registration Snapshot Time - Date/time Accurate. Slot 23404562
 '2021-03-05 19:00:53', -- Snapshot Start - Date/time Accurate. Slot?
 2950000000,            -- Voting Power Threshold -- Accurate
 100,                   -- Max Voting Power PCT - No max% threshold used in this fund.
 NULL,                  -- Insight Sharing Start - None
 '2021-01-13 21:00:00', -- Proposal Submission Start - Date/time accurate.
 '2021-01-20 21:00:00', -- Refine Proposals Start - Date/time accurate.
 '2021-01-27 21:00:00', -- Finalize Proposals Start - Date/time accurate.
 '2021-02-03 21:00:00', -- Proposal Assessment Start - None
 '2021-02-10 21:00:00', -- Assessment QA Start - Date accurate, time unknown.
 '2021-03-05 19:10:00', -- Voting Starts - Date/time not sure because very close to snapshot.
 '2021-03-29 19:00:00', -- Voting Ends - Date/time Accurate.
 '2021-04-02 19:00:00', -- Tallying Ends - Date/time Accurate.
 decode(:'block0contents','base64'),
                        -- Block 0 Data - From File
 NULL,                  -- Block 0 Hash - TODO
 0,                     -- Committee Size - No Encrypted Votes
 0                      -- Committee Threshold - No Encrypted Votes
 );

-- Free large binary file contents
\unset block0contents