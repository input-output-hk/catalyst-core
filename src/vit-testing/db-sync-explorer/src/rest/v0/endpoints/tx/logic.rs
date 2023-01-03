use crate::rest::v0::context::SharedContext;
use crate::rest::v0::errors::HandleError;
use crate::{db, TransactionConfirmation};

pub async fn get_tx_by_hash(
    hash: String,
    context: SharedContext,
) -> Result<Vec<TransactionConfirmation>, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    db::query::hash(hash, pool)
        .await
        .map(|x| x.into_iter().map(Into::into).collect())
}
