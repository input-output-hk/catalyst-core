-- Data from Catalyst Fund 2 - First Public Test

-- Data Sources see:
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2892759058/Catalyst+Fund+Cycle+Releases
-- https://docs.google.com/document/d/1dsckxH8xbGn9uoIPoFOD0I7WxWWBUlNeSZrqvU-tXvs
-- https://github.com/input-output-hk/catalyst-resources/blob/master/snapshots/snapshots.json

-- Purge all Fund 1 data before re-inserting it.
DELETE FROM event WHERE row_id = 4;

-- Load the raw Block0 Binary from the file.
-- \set block0path 'historic_data/fund_4/block0.bin'
-- \set block0contents `base64 :block0path`

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

(4, 'Catalyst Fund 4', '',
 '2021-02-17 22:00:00', -- Start Time - Date/Time accurate.
 '2021-07-04 11:00:00', -- End Time   - Date/Time accurate.
 '2021-06-11 11:00:26', -- Registration Snapshot Time - Date/time Accurate. Slot 31842935
 '2021-06-12 11:00:00', -- Snapshot Start - Date/time Unknown.
 450000000,             -- Voting Power Threshold -- Accurate
 100,                   -- Max Voting Power PCT - No max% threshold used in this fund.
 NULL,                  -- Insight Sharing Start - None
 '2021-02-24 22:00:00', -- Proposal Submission Start - Date/time accurate.
 '2021-03-03 22:00:00', -- Refine Proposals Start - Date/time accurate.
 '2021-03-10 22:00:00', -- Finalize Proposals Start - Date/time accurate.
 '2021-03-17 19:00:00', -- Proposal Assessment Start - Date/time accurate.
 '2021-03-24 19:00:00', -- Assessment QA Start - Datetime accurate.
 '2021-06-15 11:10:00', -- Voting Starts - Date/time accurate.
 '2021-06-25 11:00:00', -- Voting Ends - Date/time Accurate.
 '2021-07-04 11:00:00', -- Tallying Ends - Date/time Accurate.
 NULL, -- decode(:'block0contents','base64'), -- No Block 0 for Fund 4 located yet.
                        -- Block 0 Data - From File
 NULL,                  -- Block 0 Hash - TODO
 0,                     -- Committee Size - No Encrypted Votes
 0                      -- Committee Threshold - No Encrypted Votes
 );

-- Free large binary file contents
-- \unset block0contents