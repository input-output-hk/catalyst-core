use super::Actor;
use crate::network::wallet_state::template::Error;
use crate::wallet_state::MainnetWalletState;
use crate::CardanoWallet;
use async_trait::async_trait;
use cardano_serialization_lib::{Address, BigNum, Coin, Value};

/// Trait for retrieving information about address registrations from network
#[async_trait]
pub trait ExternalProvider {
    /// Error which can be returned on any issue with retrieving information
    type Error;

    /// Downloads `MainnetWalletState` from network
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
