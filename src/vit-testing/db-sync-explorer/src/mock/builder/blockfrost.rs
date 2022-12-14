use super::Actor;
use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::Transaction;
use mainnet_lib::{CatalystBlockFrostApi, CatalystBlockFrostApiError, TransactionBuilder};

pub async fn download_registrations_from_network(
    actors: &[Actor],
) -> Result<Vec<Transaction>, CatalystBlockFrostApiError> {
    let api = CatalystBlockFrostApi::new()?;

    let mut txs = vec![];

    for actor in actors {
        if let Actor::ExternalDelegator { address, .. } = actor {
            let address_obj = Address::from_hex(address.as_str())?;
            let stake = api.get_stake(&address_obj).await?;

            api.list_registrations_for_address(address)
                .await?
                .into_iter()
                .for_each(|metadata| {
                    txs.push(TransactionBuilder::build_transaction_with_metadata(
                        &address_obj,
                        stake,
                        &metadata,
                    ))
                });
        }
    }

    Ok(txs)
}
