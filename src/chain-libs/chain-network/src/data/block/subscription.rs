use super::{BlockId, BlockIds, Header};

/// An event sent from client to server over the block subscription stream.
#[derive(Debug)]
pub enum BlockEvent {
    /// Announcement of a new block in the chain.
    Announce(Header),
    /// Request to upload the identified blocks.
    Solicit(BlockIds),
    /// Request to push a chain of headers.
    Missing {
        /// A list of starting points known by the requester.
        /// The sender should pick the latest one.
        from: BlockIds,
        /// The identifier of the last block to send the header for.
        to: BlockId,
    },
}
