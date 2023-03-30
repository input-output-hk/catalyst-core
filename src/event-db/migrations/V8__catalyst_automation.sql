-- Catalyst Event Database

-- Voting Nodes Table - Defines nodes in the network
-- This table is looked up by hostname and event
CREATE TABLE voting_node (
    hostname TEXT NOT NULL,
    event INTEGER NOT NULL,

    pubkey TEXT NOT NULL,
    seckey TEXT NOT NULL,
    netkey TEXT NOT NULL,

    PRIMARY KEY (hostname, event),
    FOREIGN KEY(event) REFERENCES event(row_id)
);

COMMENT ON TABLE voting_node IS
'This table holds information for all nodes in the event.
It is used by nodes to self-bootstrap the blockchain.';
COMMENT ON COLUMN voting_node.hostname IS 'Unique hostname for the voting node.';
COMMENT ON COLUMN voting_node.event IS 'Unique event this node was configured for.';
COMMENT ON COLUMN voting_node.seckey IS 'Secret key from Ed25519 pair for the node. Used as the node secret.';
COMMENT ON COLUMN voting_node.pubkey IS
'Public key from Ed25519 pair for the node.
Used as consensus_leader_id when the node is a leader.';
COMMENT ON COLUMN voting_node.netkey IS
'Ed25519 secret key for the node.
Used as the node p2p topology key.';


-- Tally Committee Table - Stores data about the tally committee per voting event
-- This table is looked up by event
CREATE TABLE tally_committee (
    row_id SERIAL PRIMARY KEY,

    event INTEGER NOT NULL UNIQUE,

    pubkey TEXT NOT NULL,
    seckey TEXT NOT NULL,
    comm_pk TEXT NOT NULL,
    comm_sk TEXT NOT NULL,

    FOREIGN KEY(event) REFERENCES event(row_id)
);

COMMENT ON TABLE tally_committee IS 'Table for storing data about the tally committee per voting event.';
COMMENT ON COLUMN tally_committee.row_id IS 'Unique ID for this committee member for this event.';
COMMENT ON COLUMN tally_committee.event  IS 'The event this committee member is for.';
COMMENT ON COLUMN tally_committee.pubkey  IS 'Public key for the committee wallet.';
COMMENT ON COLUMN tally_committee.seckey  IS '';
COMMENT ON COLUMN tally_committee.comm_pk  IS '';
COMMENT ON COLUMN tally_committee.comm_sk  IS '';


-- Committee Member Table - Stores data about the tally committee members
-- This table is looked up by committee
CREATE TABLE committee_member (
    row_id SERIAL PRIMARY KEY,

    committee INTEGER NOT NULL,

    pubkey TEXT NOT NULL,
    seckey TEXT NOT NULL,
    member_pk TEXT NOT NULL,
    member_sk TEXT NOT NULL,

    FOREIGN KEY(committee) REFERENCES tally_committee(row_id)
);

COMMENT ON TABLE committee_member IS 'Table for storing data about the tally committee members.';
COMMENT ON COLUMN committee_member.row_id IS 'Unique ID for this committee member for this event.';
COMMENT ON COLUMN committee_member.committee IS 'The committe this member belongs to.';
COMMENT ON COLUMN committee_member.pubkey  IS 'Public key for the member wallet.';
COMMENT ON COLUMN committee_member.seckey  IS 'Secret key for the member wallet.';
COMMENT ON COLUMN committee_member.member_pk  IS 'Committee member public key';
COMMENT ON COLUMN committee_member.member_sk  IS 'Committee member secret key';
