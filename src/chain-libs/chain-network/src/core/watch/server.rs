use crate::data::{Block, Header};
use crate::error::Error;

use futures::prelude::*;

pub trait Watch: Send + Sync + 'static {
    type BlockSubscriptionStream: Stream<Item = Result<Block, Error>> + Send + Sync;
    type TipSubscriptionStream: Stream<Item = Result<Header, Error>> + Send + Sync;
    type SyncMultiverseStream: Stream<Item = Result<Block, Error>> + Send + Sync;
}
