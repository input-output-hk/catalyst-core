use super::Actor;
use crate::network::wallet_state::template::Error;
use crate::wallet_state::MainnetWalletState;
use crate::CardanoWallet;
use async_trait::async_trait;
use cardano_serialization_lib::address::Address;
use rand::Rng;

#[async_trait]
pub trait ExternalProvider {
    type Error;

    async fn download_state_from_network(
        &self,
        actors: &[Actor],
    ) -> Result<Vec<MainnetWalletState>, Self::Error>;
}

pub struct DummyExternalProvider;

unsafe impl Sync for DummyExternalProvider {}
unsafe impl Send for DummyExternalProvider {}

#[async_trait]
impl ExternalProvider for DummyExternalProvider {
    type Error = Error;

    async fn download_state_from_network(
        &self,
        actors: &[Actor],
    ) -> Result<Vec<MainnetWalletState>, Self::Error> {
        let mut states = Vec::new();

        for actor in actors {
            if let Actor::ExternalDelegator { address, .. } = actor {
                let mut random = rand::thread_rng();
                let wallet = CardanoWallet::new(random.gen_range(1_000..10_000));
                states.push(MainnetWalletState {
                    rep: None,
                    registration_tx: Some(wallet.generate_direct_voting_registration(0)),
                    stake: wallet.stake(),
                    stake_address: Address::from_hex(address)?,
                });
            }
        }
        Ok(states)
    }
}
