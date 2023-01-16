use crate::rest::v0::context::SharedContext;
use crate::rest::v0::errors::HandleError;
use cardano_serialization_lib::Transaction;

pub async fn submit_tx(data: Vec<u8>, context: SharedContext) -> Result<String, HandleError> {
    let transaction = Transaction::from_bytes(data)?;
    let hash = transaction.to_hex();
    context
        .write()
        .await
        .get_mock_data_provider_mut()?
        .ledger_mut()
        .push_transaction(transaction);
    Ok(hash)
}
