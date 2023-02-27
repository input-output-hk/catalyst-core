-- Your SQL goes here

-- Remove uneeded snapshot data.  Not needed for new snapshot service.

DROP INDEX IF EXISTS
    latest_registration_index;

DROP TABLE IF EXISTS
    voter_registration_category,
    voter_registration,
    stake_address_balance,
    voting_power;

-- Fix Vit-SS snapshot tables

DROP TABLE IF EXISTS
    snapshots,
    voters,
    contributions;

CREATE TABLE snapshots (
    row_id SERIAL PRIMARY KEY,
    election INTEGER NOT NULL,
    as_at TIMESTAMP NOT NULL,
    last_updated TIMESTAMP NOT NULL,
    final BOOLEAN NOT NULL,

    dbsync_snapshot_cmd  TEXT NULL,
    dbsync_snapshot_data TEXT NULL,

    drep_data TEXT NULL,

    catalyst_snapshot_cmd TEXT NULL,
    catalyst_snapshot_data TEXT NULL,

    FOREIGN KEY(election) REFERENCES Election(row_id)
);

COMMENT ON TABLE snapshots IS 'Raw snapshot data for an election.  Only the latests ansposhot per election is stored.';
COMMENT ON COLUMN snapshots.election is 'The election id this snapshot was for.';
COMMENT ON COLUMN snapshots.as_at is 'The time the snapshot was collected from dbsync (Should be the slot time the dbsync_snapshot_cmd was run against.)';
COMMENT ON COLUMN snapshots.last_updated is 'The last time the snapshot was run (Should be the real time the snapshot was started.';
COMMENT ON COLUMN snapshots.final is 'Is the snapshot Final?  No more updates will occur to this record once set.';

COMMENT ON COLUMN snapshots.dbsync_snapshot_cmd is  'The actual command run to produce the dbsync snapshot.';
COMMENT ON COLUMN snapshots.dbsync_snapshot_data is 'The raw json result stored as TEXT from the dbsync snapshot. (This is JSON data but we store as raw text to prevent any processing of it).';
COMMENT ON COLUMN snapshots.drep_data is 'The latest drep data obtained from GVC, and used in this snapshot calculation.';

COMMENT ON COLUMN snapshots.catalyst_snapshot_cmd is 'The actual command run to produce the catalyst voting power snapshot.';
COMMENT ON COLUMN snapshots.catalyst_snapshot_data is 'The raw json result stored as TEXT from the catalyst snapshot calculation. (This is JSON data but we store as raw text to prevent any processing of it).';

-- voters

CREATE TABLE voters (
    row_id SERIAL8 PRIMARY KEY,

    voting_key TEXT NOT NULL,
    snapshot_id INTEGER NOT NULL,
    voting_group TEXT NOT NULL,

    voting_power BIGINT NOT NULL,

    FOREIGN KEY(snapshot_id) REFERENCES snapshots(row_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_voter_id on voters (voting_key, voting_group, snapshot_id);

COMMENT ON TABLE voters IS 'Voting Power for every voting key.';
COMMENT ON COLUMN voters.voting_key is 'Either the voting key.';
COMMENT ON COLUMN voters.snapshot_id is 'The ID of the snapshot this record belongs to.';
COMMENT ON COLUMN voters.voting_group is 'The voter group the voter belongs to.';
COMMENT ON COLUMN voters.voting_power is 'Calculated Voting Power associated with this key.';

-- contributions

CREATE TABLE contributions (
    row_id SERIAL PRIMARY KEY,

    stake_public_key TEXT NOT NULL,
    snapshot_id INTEGER NOT NULL,

    voting_key TEXT NOT NULL,
    voting_weight INTEGER NOT NULL,
    voting_key_idx INTEGER NOT NULL,
    value BIGINT NOT NULL,

    voting_group TEXT NOT NULL,

    reward_address TEXT NULL,

    FOREIGN KEY(snapshot_id) REFERENCES snapshots(row_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_contribution_id ON contributions (stake_public_key, voting_key, voting_group, snapshot_id);

COMMENT ON TABLE contributions IS 'Individual Contributions from stake public keys to voting keys.';
COMMENT ON COLUMN contributions.row_id is 'Synthetic Unique Row Key';
COMMENT ON COLUMN contributions.stake_public_key IS 'The voters Stake Public Key';
COMMENT ON COLUMN contributions.snapshot_id IS 'The snapshot this contribution was recorded from.';

COMMENT ON COLUMN contributions.voting_key IS 'The voting key ';
COMMENT ON COLUMN contributions.voting_weight IS 'The weight this voting key gets of the total.';
COMMENT ON COLUMN contributions.voting_key_idx IS 'The index from 0 of the keys in the delegation array.';
COMMENT ON COLUMN contributions.value IS 'The amount of ADA contributed to this voting key from the stake address';

COMMENT ON COLUMN contributions.voting_group IS 'The group this contribution goes to.';

COMMENT ON COLUMN contributions.reward_address IS 'Currently Unused.  Should be the Stake Rewards address of the voter (currently unknown.)';
