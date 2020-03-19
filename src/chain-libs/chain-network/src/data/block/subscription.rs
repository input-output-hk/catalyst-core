use super::{BlockId, BlockIds, Header};

/// An event sent from client to server over the block subscription stream.
#[derive(Debug)]
pub enum BlockEvent {
    /// Announcement of a new block in the chain.
    Announce(Header),
    /// Request to upload the identified blocks.
    Solicit(BlockIds),
    /// Request to push a chain of headers.
    Missing(ChainPullRequest),
}

/// A request to send headers in the block chain sequence.
#[derive(Debug)]
pub struct ChainPullRequest {
    /// A list of starting points known by the requester.
    /// The sender should pick the latest one.
    pub from: BlockIds,
    /// The identifier of the last block to send the header for.
    pub to: BlockId,
}
