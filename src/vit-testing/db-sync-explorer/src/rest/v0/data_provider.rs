use crate::db::{Meta, Progress};
use crate::rest::v0::errors::HandleError;
use crate::{BehindDuration, TransactionConfirmation};
use async_trait::async_trait;
use std::any::Any;

#[async_trait]
pub trait DataProvider: AToAny + Send + Sync {
    async fn get_meta_info(&self) -> Result<Vec<Meta>, HandleError>;
    async fn get_interval_behind_now(&self) -> Result<BehindDuration, HandleError>;
    async fn get_sync_progress(&self) -> Result<Progress, HandleError>;
    async fn get_tx_by_hash(
        &self,
        hash: String,
    ) -> Result<Vec<TransactionConfirmation>, HandleError>;
}

pub trait AToAny: 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> AToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
