-- Data from Catalyst Fund 2 - First Funded Event

-- Data Sources see:
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2892759058/Catalyst+Fund+Cycle+Releases
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/1988329525/Fund-2
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2005532813/Release+notes
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2005860385/Go+No+Go+Dashboard
-- https://input-output.atlassian.net/wiki/spaces/VIT/pages/2133360722/Fund+2+archive
-- https://static.iohk.io/docs/catalyst/catalyst-voting-results-fund2.pdf

-- Purge all Fund 1 data before re-inserting it.
DELETE FROM event WHERE row_id = 2;

-- Load the raw Block0 Binary from the file.
\set block0path 'historic_data/fund_2/block0.bin'
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

(2, 'Catalyst Fund 2', 'First Funded Catalyst Event',
 '2020-09-23 00:00:00', -- Start Time - Date accurate, time not known.
 '2021-01-10 20:00:00', -- End Time   - Date/Time accurate.
 '2020-12-15 17:00:04', -- Registration Snapshot Time - Date/time Accurate. Slot 16485313
 '2020-12-15 17:30:00', -- Snapshot Start - Date/time Accurate. Slot?
 7950000000,            -- Voting Power Threshold -- Accurate
 100,                   -- Max Voting Power PCT - No max% threshold used in this fund.
 NULL,                  -- Insight Sharing Start - None
 '2020-09-23 00:00:00', -- Proposal Submission Start - Date accurate, time not known.
 NULL,                  -- Refine Proposals Start - Date accurate, time not known.
 '2020-10-21 23:59:59', -- Finalize Proposals Start - Date accurate, time not known.
 NULL,                  -- Proposal Assessment Start - None
 NULL,                  -- Assessment QA Start - None
 '2020-12-15 20:00:00', -- Voting Starts - Date/time Accurate.
 '2021-01-04 20:00:00', -- Voting Ends - Date/time Accurate.
 '2021-01-10 20:00:00', -- Tallying Ends - Date/time Accurate.
 decode(:'block0contents','base64'),
                        -- Block 0 Data - From File
 NULL,                  -- Block 0 Hash - TODO
 0,                     -- Committee Size - No Encrypted Votes
 0                      -- Committee Threshold - No Encrypted Votes
 );

-- Free large binary file contents
\unset block0contents