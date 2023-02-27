-- This file should undo anything in `up.sql`

CREATE TABLE voter_registration_category(
    name TEXT PRIMARY KEY,
    description TEXT
);

INSERT INTO voter_registration_category (name, description)
VALUES
    ('cip15','cip-15 registration'), -- CIP-15 style registration.
    ('cip36','cip-36 registration'), -- CIP-36 style registration.
    ('cip36d', 'cip-36 deregistration'); -- CIP-36 style deregistration.

-- Individual voter registrations

CREATE TABLE voter_registration (
    row_id SERIAL8 PRIMARY KEY,

    time TIMESTAMP,
    nonce BIGINT,

    purpose BIGINT,
    stake_pub TEXT,

    category TEXT,
    delegations JSONB,
    reward_addr TEXT,

    txn BYTEA,
    block BIGINT,

    FOREIGN KEY(category) REFERENCES voter_registration_category(name)
);

CREATE UNIQUE INDEX latest_registration_index ON voter_registration(time,nonce,purpose,stake_pub);

COMMENT ON TABLE voter_registration IS 'Every voter registration made on cardano.';
COMMENT ON COLUMN voter_registration.row_id IS 'Synthetic Unique ID for a particular voter registration record.';
COMMENT ON COLUMN voter_registration.time IS 'The time the voter registration was made, derived from the block the registration was in.';
COMMENT ON COLUMN voter_registration.delegations IS
'The JSON equivalent encoding of the voting key or delegations Array:
  IF category == "cip15":
    {"key":"<voting key>"}
    (See cip-15 <https://cips.cardano.org/cips/cip15/> for details.)
  IF category == "cip36":
    {"delegation":[["<voting key>",weight],...]}
    (See cip-36 <https://cips.cardano.org/cips/cip36/> for details.)
  IF category == "cip36d":
    {}
    (See cip-36 <https://cips.cardano.org/cips/cip36/> for details.)';
COMMENT ON COLUMN voter_registration.stake_pub IS 'A stake address for the network that this transaction is submitted to (to point to the Ada that is being delegated).';
COMMENT ON COLUMN voter_registration.reward_addr IS 'A Shelley address discriminated for the same network this transaction is submitted to to receive rewards.';
COMMENT ON COLUMN voter_registration.nonce IS 'A nonce that identifies that most recent delegation. In case multiple registrations make it into the same block.';
COMMENT ON COLUMN voter_registration.purpose IS 'A non-negative integer that indicates the purpose of the vote.';
COMMENT ON COLUMN voter_registration.txn IS 'The raw transaction from the blockchain, including its witness.';
COMMENT ON COLUMN voter_registration.block IS 'The block this transaction is located in on the cardano blockchain.';
COMMENT ON INDEX latest_registration_index IS 'Index to find latest registrations per unique stake_pub key faster.';

-- Every balance change to every stake address, ever.

CREATE TABLE stake_address_balance (
    row_id SERIAL8 PRIMARY KEY,
    time TIMESTAMP,
    block  BIGINT,

    public_key TEXT,
    balance NUMERIC,
    unpaid_rewards NUMERIC
);

COMMENT ON TABLE stake_address_balance IS 'The balance of a particular stake address at a particular point in time. Note, this table catches ALL stake addresses, not only those registered.';
COMMENT ON COLUMN stake_address_balance.row_id IS 'Synthetic record ID of the balance update.';
COMMENT ON COLUMN stake_address_balance.time IS 'The time the stake address balance changed to the value in this record.';
COMMENT ON COLUMN stake_address_balance.block IS
'The block number on cardano where this balance change occurred.
If there were multiple changes to the stake address balance made in the same block, then this record is the result of
all changes at the end of the block, after all transaction are processed.';
COMMENT ON COLUMN stake_address_balance.public_key IS 'The stake public key address who''s value changed.';
COMMENT ON COLUMN stake_address_balance.balance IS 'The ADA.LOVELACE balance of the stake address, at this time.';
COMMENT ON COLUMN stake_address_balance.unpaid_rewards IS 'The ADA.LOVELACE balance of all unpaid rewards associated with this public stake key address.';

-- snapshot - Etched in stone snapshot of voting power for each election.

CREATE TABLE voting_power (
    row_id SERIAL8 PRIMARY KEY,
    election INTEGER,

    voting_key TEXT,
    power NUMERIC,

    FOREIGN KEY(election) REFERENCES Election(row_id)
)

-- Fix old vit-ss schema

-- These tables are not updated to the new schema and still need analysis.
-- They are a copy of the latest vit-ss database table definitions.

DROP TABLE IF EXISTS
    snapshots,
    voters,
    contributions;

-- TODO, Shouldn't this be related to the other tables?

COMMENT ON TABLE votes IS 'All Votes. VIT-SS Compatibility Table, to be replaced when analysis completed.';
COMMENT ON COLUMN votes.fragment_id is 'TODO';
COMMENT ON COLUMN votes.caster is 'TODO';
COMMENT ON COLUMN votes.proposal is 'TODO';
COMMENT ON COLUMN votes.voteplan_id is 'TODO';
COMMENT ON COLUMN votes.time is 'TODO';
COMMENT ON COLUMN votes.choice is 'TODO';
COMMENT ON COLUMN votes.raw_fragment is 'TODO';

-- snapshots

CREATE TABLE snapshots (
    tag TEXT PRIMARY KEY,
    last_updated BIGINT NOT NULL
);

COMMENT ON TABLE snapshots IS 'Something to do with snapshots. VIT-SS Compatibility Table, to be replaced when analysis completed.';
COMMENT ON COLUMN snapshots.last_updated is 'TODO';

-- voters

CREATE TABLE voters (
    voting_key TEXT NOT NULL,
    voting_power BIGINT NOT NULL,
    voting_group TEXT NOT NULL,
    snapshot_tag TEXT NOT NULL,

    PRIMARY KEY(voting_key, voting_group, snapshot_tag),
    FOREIGN KEY(snapshot_tag) REFERENCES snapshots(tag) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_voter_id on voters (voting_key, voting_group, snapshot_tag);

COMMENT ON TABLE voters IS 'Voters table. VIT-SS Compatibility Table, to be replaced when analysis completed.';
COMMENT ON COLUMN voters.voting_key is 'TODO';
COMMENT ON COLUMN voters.voting_power is 'TODO';
COMMENT ON COLUMN voters.voting_group is 'TODO';
COMMENT ON COLUMN voters.snapshot_tag is 'TODO';

-- contributions

CREATE TABLE contributions (
    row_id SERIAL PRIMARY KEY,

    stake_public_key TEXT NOT NULL,
    voting_key TEXT NOT NULL,
    voting_group TEXT NOT NULL,
    snapshot_tag TEXT NOT NULL,

    reward_address TEXT NOT NULL,
    value BIGINT NOT NULL,

    FOREIGN KEY(snapshot_tag) REFERENCES snapshots(tag) ON DELETE CASCADE
);

CREATE UNIQUE INDEX unique_contribution_id ON contributions (stake_public_key, voting_key, voting_group, snapshot_tag);

COMMENT ON TABLE contributions IS 'TODO';
COMMENT ON COLUMN contributions.row_id is 'Synthetic Unique Row Key';
COMMENT ON COLUMN contributions.stake_public_key IS 'TODO';
COMMENT ON COLUMN contributions.voting_key IS 'TODO';
COMMENT ON COLUMN contributions.voting_group IS 'TODO';
COMMENT ON COLUMN contributions.snapshot_tag IS 'TODO';
COMMENT ON COLUMN contributions.reward_address IS 'TODO';
COMMENT ON COLUMN contributions.value IS 'TODO';
