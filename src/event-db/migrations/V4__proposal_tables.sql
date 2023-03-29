-- Catalyst Event Database

-- Proposals Table

CREATE TABLE proposal
(
    row_id SERIAL PRIMARY KEY,
    id INTEGER NOT NULL UNIQUE,
    challenge INTEGER NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    category TEXT NOT NULL,
    public_key TEXT NOT NULL,
    funds BIGINT NOT NULL,
    url TEXT NOT NULL,
    files_url TEXT NOT NULL,
    impact_score BIGINT NOT NULL,

    extra JSONB,

    proposer_name TEXT NOT NULL,
    proposer_contact TEXT NOT NULL,
    proposer_url TEXT NOT NULL,
    proposer_relevant_experience TEXT NOT NULL,
    bb_proposal_id BYTEA,

    bb_vote_options TEXT,

    FOREIGN KEY(challenge) REFERENCES challenge(id),
    FOREIGN KEY(bb_vote_options) REFERENCES vote_options(challenge)
);

CREATE UNIQUE INDEX proposal_index ON proposal(row_id, challenge);


COMMENT ON TABLE proposal IS 'All Proposals for the current fund.';
COMMENT ON COLUMN proposal.row_id IS 'Synthetic Unique Key';
COMMENT ON COLUMN proposal.id IS 'Actual Proposal Unique ID';
COMMENT ON COLUMN proposal.challenge IS 'The Challenge this proposal falls under.';
COMMENT ON COLUMN proposal.title IS 'Brief title of the proposal.';
COMMENT ON COLUMN proposal.summary IS 'A Summary of the proposal to be implemented.';
COMMENT ON COLUMN proposal.public_key IS '???';
COMMENT ON COLUMN proposal.funds IS 'How much funds (in the currency of the fund)';
COMMENT ON COLUMN proposal.url IS 'A URL with supporting information for the proposal.';
COMMENT ON COLUMN proposal.files_url IS 'A URL link to relevant files supporting the proposal.';
COMMENT ON COLUMN proposal.impact_score IS 'The Impact score assigned to this proposal by the Assessors.';
COMMENT ON COLUMN proposal.proposer_name IS 'The proposers name.';
COMMENT ON COLUMN proposal.proposer_contact IS 'Contact details for the proposer.';
COMMENT ON COLUMN proposal.proposer_url IS 'A URL with details of the proposer.';
COMMENT ON COLUMN proposal.proposer_relevant_experience IS 'A freeform  string describing the proposers experience relating to their capability to implement the proposal.';
COMMENT ON COLUMN proposal.bb_proposal_id IS 'The ID used by the voting ledger (bulletin board) to refer to this proposal.';
COMMENT ON COLUMN proposal.bb_vote_options IS 'The selectable options by the voter.';
COMMENT ON COLUMN proposal.extra IS
'Extra data about the proposal.
 The types of extra data are defined by the proposal type and are not enforced.
 Extra Fields for `native` challenges:
    NONE.

 Extra Fields for `simple` challenges:
    "solution" : <text> - The Solution to the challenge.

 Extra Fields for `community choice` challenge:
    "brief"      : <text> - Brief explanation of a proposal.
    "importance" : <text> - The importance of the proposal.
    "goal"       : <text> - The goal of the proposal is addressed to meet.
    "metrics"    : <text> - The metrics of the proposal or how success will be determined.';

-- community advisor reviews

-- I feel like these ratings and notes should be in a  general json field to
-- suit adaptability without needing schema changes.

CREATE TABLE community_advisors_review (
  row_id SERIAL PRIMARY KEY,
  proposal_id INTEGER NOT NULL,
  assessor VARCHAR NOT NULL,
  impact_alignment_rating_given INTEGER,
  impact_alignment_note VARCHAR,
  feasibility_rating_given INTEGER,
  feasibility_note VARCHAR,
  auditability_rating_given INTEGER,
  auditability_note VARCHAR,
  ranking INTEGER,

  FOREIGN KEY (proposal_id) REFERENCES proposal(id) ON DELETE CASCADE
);

COMMENT ON TABLE community_advisors_review IS 'All Reviews.';
COMMENT ON COLUMN community_advisors_review.row_id IS 'Synthetic Unique Key.';
COMMENT ON COLUMN community_advisors_review.proposal_id IS 'The Proposal this review is for.';
COMMENT ON COLUMN community_advisors_review.assessor IS 'Assessors Anonymized ID';
COMMENT ON COLUMN community_advisors_review.impact_alignment_rating_given IS 'The  numeric rating assigned to the proposal by the assessor.';
COMMENT ON COLUMN community_advisors_review.impact_alignment_note IS 'A note about why the impact rating was given.';
COMMENT ON COLUMN community_advisors_review.feasibility_rating_given IS 'The numeric feasibility rating given.';
COMMENT ON COLUMN community_advisors_review.feasibility_note IS 'A note about why the feasibility rating was given.';
COMMENT ON COLUMN community_advisors_review.auditability_rating_given IS 'The numeric auditability rating given.';
COMMENT ON COLUMN community_advisors_review.auditability_note IS 'A note about the auditability rating given.';
COMMENT ON COLUMN community_advisors_review.ranking IS 'Numeric  Measure of quality of this review according to veteran community advisors.';
