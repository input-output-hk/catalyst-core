-- Catalyst Event Database

-- Voting Power Snapshot Table

CREATE TABLE snapshot (
    row_id SERIAL PRIMARY KEY,
    event INTEGER NOT NULL UNIQUE,
    as_at TIMESTAMP NOT NULL,
    last_updated TIMESTAMP NOT NULL,
    final BOOLEAN NOT NULL,

    dbsync_snapshot_cmd    TEXT NULL,
    dbsync_snapshot_params JSONB NULL,

    dbsync_snapshot_data TEXT NULL,

    drep_data TEXT NULL,

    catalyst_snapshot_cmd TEXT NULL,
    catalyst_snapshot_params JSONB NULL,
    catalyst_snapshot_data TEXT NULL,

    FOREIGN KEY(event) REFERENCES event(row_id)
);

COMMENT ON TABLE snapshot IS 'Raw snapshot data for an event.  Only the latests snapshot per event is stored.';
COMMENT ON COLUMN snapshot.event is 'The event id this snapshot was for.';
COMMENT ON COLUMN snapshot.as_at is 'The time the snapshot was collected from dbsync (Should be the slot time the dbsync_snapshot_cmd was run against.)';
COMMENT ON COLUMN snapshot.last_updated is 'The last time the snapshot was run (Should be the real time the snapshot was started.';
COMMENT ON COLUMN snapshot.final is 'Is the snapshot Final?  No more updates will occur to this record once set.';

COMMENT ON COLUMN snapshot.dbsync_snapshot_cmd is     'The name of the command run to collect the snapshot from dbsync.';
COMMENT ON COLUMN snapshot.dbsync_snapshot_params is  'The parameters passed to the command, each parameter is a key and its value is the value of the parameter.';
COMMENT ON COLUMN snapshot.dbsync_snapshot_data is 'The raw json result stored as TEXT from the dbsync snapshot. (This is JSON data but we store as raw text to prevent any processing of it).';

COMMENT ON COLUMN snapshot.drep_data is 'The latest drep data obtained from GVC, and used in this snapshot calculation.  Should be in a form directly usable by the `catalyst_snapshot_cmd`';

COMMENT ON COLUMN snapshot.catalyst_snapshot_cmd is  'The actual name of the command run to produce the catalyst voting power snapshot.';
COMMENT ON COLUMN snapshot.dbsync_snapshot_params is 'The parameters passed to the command, each parameter is a key and its value is the value of the parameter.';
COMMENT ON COLUMN snapshot.catalyst_snapshot_data is 'The raw json result stored as TEXT from the catalyst snapshot calculation. (This is JSON data but we store as raw text to prevent any processing of it).';

-- voters

CREATE TABLE voter (
    row_id SERIAL8 PRIMARY KEY,

    voting_key TEXT NOT NULL,
    snapshot_id INTEGER NOT NULL,
    voting_group TEXT NOT NULL,

    voting_power BIGINT NOT NULL,

    FOREIGN KEY(snapshot_id) REFERENCES snapshot(row_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_voter_id on voter (voting_key, voting_group, snapshot_id);

COMMENT ON TABLE voter IS 'Voting Power for every voting key.';
COMMENT ON COLUMN voter.voting_key is 'Either the voting key.';
COMMENT ON COLUMN voter.snapshot_id is 'The ID of the snapshot this record belongs to.';
COMMENT ON COLUMN voter.voting_group is 'The voter group the voter belongs to.';
COMMENT ON COLUMN voter.voting_power is 'Calculated Voting Power associated with this key.';

-- contributions

CREATE TABLE contribution (
    row_id SERIAL PRIMARY KEY,

    stake_public_key TEXT NOT NULL,
    snapshot_id INTEGER NOT NULL,

    voting_key TEXT NOT NULL,
    voting_weight INTEGER NOT NULL,
    voting_key_idx INTEGER NOT NULL,
    value BIGINT NOT NULL,

    voting_group TEXT NOT NULL,

    reward_address TEXT NULL,

    FOREIGN KEY(snapshot_id) REFERENCES snapshot(row_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_contribution_id ON contribution (stake_public_key, voting_key, voting_group, snapshot_id);

COMMENT ON TABLE contribution IS 'Individual Contributions from stake public keys to voting keys.';
COMMENT ON COLUMN contribution.row_id is 'Synthetic Unique Row Key';
COMMENT ON COLUMN contribution.stake_public_key IS 'The voters Stake Public Key';
COMMENT ON COLUMN contribution.snapshot_id IS 'The snapshot this contribution was recorded from.';

COMMENT ON COLUMN contribution.voting_key IS 'The voting key ';
COMMENT ON COLUMN contribution.voting_weight IS 'The weight this voting key gets of the total.';
COMMENT ON COLUMN contribution.voting_key_idx IS 'The index from 0 of the keys in the delegation array.';
COMMENT ON COLUMN contribution.value IS 'The amount of ADA contributed to this voting key from the stake address';

COMMENT ON COLUMN contribution.voting_group IS 'The group this contribution goes to.';

COMMENT ON COLUMN contribution.reward_address IS 'Currently Unused.  Should be the Stake Rewards address of the voter (currently unknown.)';
