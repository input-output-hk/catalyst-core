-- Catalyst Event Database


-- Event Table - Defines each voting or decision event

CREATE TABLE event
(
    row_id SERIAL PRIMARY KEY,

    name TEXT NOT NULL,
    description TEXT NOT NULL,

    registration_snapshot_time TIMESTAMP,
    voting_power_threshold BIGINT,
    max_voting_power_pct NUMERIC(6,3) CONSTRAINT percentage CHECK (max_voting_power_pct <= 100),

    start_time TIMESTAMP,
    end_time TIMESTAMP,

    insight_sharing_start TIMESTAMP,
    proposal_submission_start TIMESTAMP,
    refine_proposals_start TIMESTAMP,
    finalize_proposals_start TIMESTAMP,
    proposal_assessment_start TIMESTAMP,
    assessment_qa_start TIMESTAMP,
    snapshot_start TIMESTAMP,
    voting_start TIMESTAMP,
    voting_end TIMESTAMP,
    tallying_end TIMESTAMP,

    extra JSONB
);

CREATE UNIQUE INDEX event_name_idx ON event(name);

COMMENT ON TABLE event IS 'The basic parameters of each voting/decision event.';
COMMENT ON COLUMN event.row_id IS 'Synthetic Unique ID for each event.';
COMMENT ON COLUMN event.name IS 'The name of the event, for example "Fund9" or "SVE1"';
COMMENT ON COLUMN event.description IS 'A detailed description of the purpose of the event.  For example, the Funds "Goal".';
COMMENT ON COLUMN event.registration_snapshot_time IS 'The Time (UTC) Registrations are taken from Cardano main net,  registrations after this date are not valid for voting on the event. NULL = Not yet defined or Not Applicable.';
COMMENT ON COLUMN event.voting_power_threshold IS 'The Minimum number of Lovelace staked at the time of snapshot, to be eligible to vote. NULL = Not yet defined.';
COMMENT ON COLUMN event.start_time IS 'The time (UTC) the event starts.  NULL = Not yet defined.';
COMMENT ON COLUMN event.end_time IS 'The time (UTC) the event ends.  NULL = Not yet defined.';
COMMENT ON COLUMN event.insight_sharing_start IS 'TODO';
COMMENT ON COLUMN event.proposal_submission_start IS 'The Time (UTC) proposals can start to be submitted for the event.  NULL = Not yet defined, or Not applicable.';
COMMENT ON COLUMN event.refine_proposals_start IS 'TODO';
COMMENT ON COLUMN event.finalize_proposals_start IS 'The Time (UTC) when all proposals must be finalized by. NULL = Not yet defined, or Not applicable.';
COMMENT ON COLUMN event.proposal_assessment_start IS 'The Time (UTC) when PA Assessors can start assessing proposals. NULL = Not yet defined, or Not applicable.';
COMMENT ON COLUMN event.assessment_qa_start IS 'The Time (UTC) when vPA Assessors can start assessing assessments. NULL = Not yet defined, or Not applicable.';
COMMENT ON COLUMN event.snapshot_start IS 'The Time (UTC) when the voting power of registered stake addresses is taken from main net. NULL = Not yet defined.';
COMMENT ON COLUMN event.voting_start IS 'The earliest time that registered wallets with sufficient voting power can place votes in the event. NULL = Not yet defined.';
COMMENT ON COLUMN event.voting_end IS 'The latest time that registered wallets with sufficient voting power can place votes in the event. NULL = Not yet defined.';
COMMENT ON COLUMN event.tallying_end IS 'The latest time that tallying the event can complete by. NULL = Not yet defined.';
COMMENT ON COLUMN event.extra IS 'Json Map defining event specific extra data. NULL = Not yet defined. "url"."results" = a results URL, "url"."survey" = a survey URL, others can be defined as required.';
