use crate::rest::v0::context::SharedContext;
use crate::rest::v0::errors::HandleError;
use crate::TransactionConfirmation;

pub async fn get_tx_by_hash(
    hash: String,
    context: SharedContext,
) -> Result<Vec<TransactionConfirmation>, HandleError> {
    context.read().await.provider().get_tx_by_hash(hash).await
}
