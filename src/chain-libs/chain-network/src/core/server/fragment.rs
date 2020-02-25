use crate::data::{Fragment, FragmentIds};
use crate::error::Error;
use async_trait::async_trait;
use futures::prelude::*;

/// Interface for the blockchain node service implementation responsible for
/// exchanging fragments that make up a future block.
#[async_trait]
pub trait FragmentService {
    /// The type of an asynchronous stream that provides blocks in
    /// response to `get_fragments` method.
    type GetFragmentsStream: Stream<Item = Result<Fragment, Error>> + Send + Sync;

    /// Serves a request to retrieve blocks identified by the list of `ids`
    /// Resloves to a stream of blocks to send to the remote client peer.
    async fn get_fragments(&self, ids: FragmentIds) -> Result<Self::GetFragmentsStream, Error>;
}
