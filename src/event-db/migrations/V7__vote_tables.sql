-- Catalyst Event Database - VIT-SS Compatibility

-- These tables need more analysis and may not be final.

-- vote storage (replicates on-chain data for easy querying)

CREATE TABLE vote (
    row_id SERIAL8 PRIMARY KEY,
    event          INTEGER,
    fragment_id    TEXT,

    voter          INTEGER NOT NULL,
    challenge      INTEGER NOT NULL,
    proposal       INTEGER NULL,
    voteplan       INTEGER NOT NULL,

    cast_at        TIMESTAMP NOT NULL,

    choice         SMALLINT NULL,
    raw_fragment   TEXT     NOT NULL,

    FOREIGN KEY(event)               REFERENCES event(row_id),
    FOREIGN KEY(voter)               REFERENCES voter(row_id),
    FOREIGN KEY(challenge)           REFERENCES challenge(row_id),
    FOREIGN KEY(proposal)            REFERENCES proposal(row_id),
    FOREIGN KEY(challenge, proposal) REFERENCES proposal(challenge, row_id),
    FOREIGN KEY(voteplan)            REFERENCES voteplan(row_id)
);

CREATE UNIQUE INDEX vote_event_idx ON vote(event,fragment_id);

COMMENT ON TABLE vote IS 'All Votes.';
COMMENT ON COLUMN vote.fragment_id is 'Unique ID of this Vote';
COMMENT ON COLUMN vote.voter is 'Reference to the Voter who cast the vote';
COMMENT ON COLUMN vote.challenge is 'Reference to the Challenge the vote was for.';
COMMENT ON COLUMN vote.proposal is
'Reference to the Proposal the vote was for.
May be NULL if this vote covers ALL proposals in the challenge.';
COMMENT ON COLUMN vote.voteplan is 'The Voteplan this vote was cast in';
COMMENT ON COLUMN vote.cast_at is 'When this vote was recorded as properly cast';
COMMENT ON COLUMN vote.choice is 'If a public vote, the choice of the vote, otherwise NULL.';
COMMENT ON COLUMN vote.raw_fragment is 'The raw vote record.';
