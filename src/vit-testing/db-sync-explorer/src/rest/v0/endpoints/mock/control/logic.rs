use crate::mock::{build_block0, Providers};
use mainnet_lib::{InMemoryDbSync, Ledger};

use crate::rest::v0::endpoints::mock::control::request::ResetRequest;
use crate::rest::v0::errors::HandleError;

pub async fn reset_data(
    reset_request: ResetRequest,
    provider_config: Providers,
) -> Result<(Ledger, InMemoryDbSync), HandleError> {
    let mut db_sync = InMemoryDbSync::default();
    let block0 = build_block0(reset_request.actors, provider_config).await?;

    let ledger = Ledger::new(block0.clone());
    db_sync.on_block_propagation(&block0.block);

    Ok((ledger, db_sync))
}
