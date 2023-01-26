use async_trait::async_trait;
use cardano_serialization_lib::address::Address;
use mainnet_lib::wallet_state::{Actor, ExternalProvider, MainnetWalletState};
use mainnet_lib::{CatalystBlockFrostApi, CatalystBlockFrostApiError, TransactionBuilder};

pub struct BlockfrostProvider {}

#[async_trait]
impl ExternalProvider for BlockfrostProvider {
    type Error = CatalystBlockFrostApiError;

    async fn download_state_from_network(
        &self,
        actors: &[Actor],
    ) -> Result<Vec<MainnetWalletState>, Self::Error> {
        let api = CatalystBlockFrostApi::new()?;

        let mut states = vec![];

        for actor in actors {
            if let Actor::ExternalDelegator { address, .. } = actor {
                let address_obj = Address::from_hex(address.as_str())?;
                let stake = api.get_stake(&address_obj).await?;

                let registration_tx = api
                    .list_registrations_for_address(address)
                    .await?
                    .into_iter()
                    .map(|metadata| {
                        TransactionBuilder::build_transaction_with_metadata(
                            &address_obj,
                            stake,
                            &metadata,
                        )
                    })
                    .last();

                states.push(MainnetWalletState {
                    rep: None,
                    registration_tx,
                    stake,
                    stake_address: Address::from_hex(address)?,
                });
            }
        }

        Ok(states)
    }
}
