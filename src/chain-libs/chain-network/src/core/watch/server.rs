use crate::data::{Block, BlockIds, Header};
use crate::error::Error;

use async_trait::async_trait;
use futures::prelude::*;

#[async_trait]
pub trait Watch: Send + Sync + 'static {
    type BlockSubscriptionStream: Stream<Item = Result<Block, Error>> + Send + Sync;

    async fn block_subscription(&self) -> Result<Self::BlockSubscriptionStream, Error>;

    type TipSubscriptionStream: Stream<Item = Result<Header, Error>> + Send + Sync;

    async fn tip_subscription(&self) -> Result<Self::TipSubscriptionStream, Error>;

    type SyncMultiverseStream: Stream<Item = Result<Block, Error>> + Send + Sync;

    async fn sync_multiverse(&self, from: BlockIds) -> Result<Self::SyncMultiverseStream, Error>;
}
