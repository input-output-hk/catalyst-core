use crate::mock::Providers;
use cardano_serialization_lib::Transaction;
use mainnet_lib::wallet_state::build_default;
use mainnet_lib::{Block0, BlockBuilder, InMemoryDbSync, Ledger};

use crate::rest::v0::endpoints::mock::control::request::ResetRequest;
use crate::rest::v0::errors::HandleError;

pub async fn reset_data(
    reset_request: ResetRequest,
    _provider_config: Providers,
) -> Result<(Ledger, InMemoryDbSync), HandleError> {
    let mut db_sync = InMemoryDbSync::default();
    let wallets = build_default(reset_request.actors).await?;
    let txs: Vec<Transaction> = wallets
        .into_iter()
        .filter_map(|x| x.registration_tx)
        .collect();
    let block0 = Block0 {
        block: BlockBuilder::next_block(None, &txs),
        settings: Default::default(),
    };

    let ledger = Ledger::new(block0.clone());
    db_sync.on_block_propagation(&block0.block);

    Ok((ledger, db_sync))
}
