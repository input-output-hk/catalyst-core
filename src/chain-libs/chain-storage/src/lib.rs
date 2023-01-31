//! Block storage crate.
//!
//! # Data model
//!
//! This storage is designed to be independent from a particular block strucure.
//! At the same time, since it is designed specifically for block chains, it
//! handles the minimum amount of data required to maintain consistency. Such
//! data includes:
//!
//! * Block ID
//! * ID of the parent block
//! * Chain length
//!
//! This data is provided alongside a block in the `BlockInfo` structure.
//!
//! # Volatile + permanent storage model
//!
//! Since blockchain can be branching extensively, this library provides the
//! mechanism to have multiple blockchain branches. However, doing so for all
//! block screates a lot of overhead for maintaining volatile storage. At the
//! same time, branches usually die out pretty fast.
//!
//! Given that, the storage is separated into two parts:
//!
//! * Volatile storage should be used for the part of a blockchain where a
//!   developer may need access to different branches.
//! * Permanent storage for the part of the blockchain that is guaranteed not to
//!   change anymore.
//!
//! ## Moving blocks to the permanent store.
//!
//! if you have this block structure and call
//! `store.flush_to_permanent_store(block 2 id)`, then `block 1` and `block 2`
//! will be moved to the permanent storage. if you call
//! `store.flush_to_permanent_store(block 3 id)`, `block 3` will also be moved
//! to the permanent store, but `block 3'` will still exist in the volatile store.
//! note that if you call `store.get_blocks_by_chain_length(3)` only `block 3`
//! (which is in the permanent store) will be returned; and you cannot call
//! `store.flush_to_permanent_store(block 3' id)` now.
//!
//! __fig.1 - note that root does not actually exist, this is an id referredby__
//! __the first block in the chain__
//!
//! ```text
//! +---------+       +---------+
//! |         |       |         |
//! | block 4 |       | block 4'|
//! |         |       |         |
//! +---------+       +---------+
//!      |                 |
//!      |                 |
//!      v                 v
//! +---------+       +---------+
//! |         |       |         |
//! | block 3 |       | block 3'|
//! |         |       |         |
//! +---------+       +---------+
//!      |                 |
//!      |                 |
//!      v                 |
//! +---------+            |
//! |         |            |
//! | block 2 +<-----------+
//! |         |
//! +---------+
//!      |
//!      |
//!      v
//! +---------+
//! |         |
//! | block 1 |
//! |         |
//! +---------+
//!      |
//!      |
//!      v
//! +---------+
//! |         |
//! |  (root) |
//! |         |
//! +---------+
//! ```
//!
//! ## Removing stale branches
//!
//! If you want to clean up branches, do the following:
//!
//! * Call `store.get_tips_ids()`, in our example it will return
//!   `[Block 4 id, Block 4' id]`;
//! * Determine which branches do you want to remove.
//! * Call, for example, `store.prune_branch(Block 4' id)`.
//!
//! ## Performance benefits of permanent storage
//!
//! Since blocks in the permanent storage are stored just one after another (the
//! structure is `block_length.to_le_bytes() ++ block_bytes` and a file with
//! references to blocks in the order they were added to the storage), it allows
//! for the following scenarios:
//!
//! * Very fast block iteration
//! * Transferring a portion of the data file over the network without locking
//!   it.
//! * O(1) access by chain length.
//!
//! __fig. 2 - permanent storage structure__
//!
//! ```text
//! store                  block no. index
//!
//! +--------------+       +-------------+
//! |              |       |             |
//! | block1       |       | block1 pos  |
//! |              |       |             |
//! +--------------+       +-------------+
//! |              |       |             |
//! | block2       |       | block2 pos  |
//! |              |       |             |
//! +--------------+       +-------------+
//! |              |       |             |
//! | ...          |       | ...         |
//! |              |       |             |
//! +--------------+       +-------------+
//! |              |       |             |
//! | blockn       |       | blockn pos  |
//! |              |       |             |
//! +--------------+       +-------------+
//! ```
//!
//! # Storage directory layout
//!
//! ```text
//! store
//! ├── permanent       - permanent storage directory
//! │   └── flatfile    - storage file that can be transferred over the network
//! └── volatile        - volatile storage
//! ```

mod block_info;
mod block_store;
mod error;
mod iterator;
mod permanent_store;
#[cfg(any(test, feature = "with-bench"))]
pub mod test_utils;
#[cfg(test)]
mod tests;
mod value;

pub use block_info::BlockInfo;
pub use block_store::BlockStore;
pub use error::{ConsistencyFailure, Error};
pub use iterator::StorageIterator;
pub use value::Value;
