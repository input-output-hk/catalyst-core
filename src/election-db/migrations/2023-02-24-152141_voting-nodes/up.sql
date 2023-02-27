-- Metadata For Voting Nodes

-- Voting Nodes Table - Defines nodes in the network
-- This table is looked up by hostname
CREATE TABLE voting_nodes (
    hostname NAME PRIMARY KEY,
    pubkey TEXT NOT NULL,
    seckey TEXT NOT NULL,
    netkey TEXT NOT NULL
);

COMMENT ON TABLE voting_nodes IS 'This table holds information for all nodes in the election, and it is used by nodes to self-bootstrap the blockchain.';
COMMENT ON COLUMN voting_nodes.hostname IS 'Unique hostname for the voting node.';
COMMENT ON COLUMN voting_nodes.seckey IS 'Secret key from Ed25519 pair for the node. Used as the node secret.';
COMMENT ON COLUMN voting_nodes.pubkey IS 'Public key from Ed25519 pair for the node. Used as consensus_leader_id when the node is a leader.';
COMMENT ON COLUMN voting_nodes.netkey IS 'Ed25519 secret key for the node. Used as the node p2p topology key.';
