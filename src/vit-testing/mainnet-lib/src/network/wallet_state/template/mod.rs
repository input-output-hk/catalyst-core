mod actor;
mod arbitrary;
mod external_providers;

use super::MainnetWalletState;
use crate::CardanoWallet;
pub use actor::Actor;
pub use arbitrary::generate_arbitrary_delegator;
use chain_crypto::PublicKeyFromStrError;
pub use external_providers::{DummyExternalProvider, ExternalProvider};
use jormungandr_lib::crypto::account::Identifier;

/// Builds collection of `MainnetWalletState` structs based on default configuration
///
/// # Errors
///
/// On error from dummy provider or incorrect actor configuration
pub async fn build_default(actors: Vec<Actor>) -> Result<Vec<MainnetWalletState>, Error> {
    build(actors, &DummyExternalProvider).await
}

/// Builds collection of `MainnetWalletState` structs based on configuration and external provider
/// which will provide information about existing wallets
/// # Errors
///
/// On error from provider or incorrect actor configuration
pub async fn build(
    actors: Vec<Actor>,
    external_provider: &(dyn ExternalProvider<Error = Error> + Send + Sync),
) -> Result<Vec<MainnetWalletState>, Error> {
    let generated_reps_def: Vec<&Actor> = actors
        .iter()
        .filter(|x| matches!(x, Actor::GeneratedRep { .. }))
        .collect();
    let generated_delegation_def: Vec<&Actor> = actors
        .iter()
        .filter(|x| matches!(x, Actor::GeneratedDelegator { .. }))
        .collect();
    let external_reps_def: Vec<&Actor> = actors
        .iter()
        .filter(|x| matches!(x, Actor::ExternalRep { .. }))
        .collect();
    let generated_reps: Vec<(String, CardanoWallet)> = generated_reps_def
        .iter()
        .map(|x| arbitrary::generate_arbitrary_representative(x))
        .collect();

    let mut voting_keys = vec![];

    voting_keys.extend(
        generated_reps
            .iter()
            .map(|(name, wallet)| (name, wallet.catalyst_public_key())),
    );

    for external_rep in external_reps_def {
        match external_rep {
            Actor::ExternalRep { name, voting_key } => {
                voting_keys.push((
                    name,
                    Identifier::from_hex(voting_key).map_err(Error::CannotGetVotingKey)?,
                ));
            }
            _ => {
                return Err(Error::Internal(
                    "collection of external reps should be already filtered out".to_string(),
                ))
            }
        }
    }

    let mut delegators: Vec<MainnetWalletState> = Vec::new();

    for actor in generated_delegation_def {
        delegators.push(generate_arbitrary_delegator(actor, &voting_keys)?);
    }

    delegators.extend(
        external_provider
            .download_state_from_network(&actors)
            .await?,
    );

    Ok(delegators)
}

/// Wallet state template builder error
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),
    /// Cannot get voting key
    #[error("cannot convert voting key")]
    CannotGetVotingKey(#[from] PublicKeyFromStrError),
    /// Arbitrary error
    #[error(transparent)]
    Arbitrary(#[from] arbitrary::Error),
    /// Arbitrary error
    #[error(transparent)]
    CannotParseAddress(#[from] cardano_serialization_lib::error::JsError),
}

#[cfg(test)]
mod tests {
    use crate::network::wallet_state::template::{
        actor::delegator, actor::registration, actor::representative, build_default,
    };
    use crate::{Block0, BlockBuilder, Ledger, Settings};
    use cardano_serialization_lib::Transaction;

    #[tokio::test]
    pub async fn test() {
        let actors = vec![
            representative("alice").with_ada(1000).try_into().unwrap(),
            representative("bob").with_key("e45f2426ffdcf4e819ca38c15adb396d9c1519ffa7239f11ae8cd79f15bc38b5").try_into().unwrap(),
            delegator("clarice").with_registration(registration().at_slot(1).with_targets(vec![("bob",1)]).try_into().unwrap()).with_ada(100).try_into().unwrap(),
            delegator("david").with_address("addr_test1qqpjfknn6azgwjs0u4x7cjmng8x7m2uve0eyqk2vafa9ghc68ap070sg8any3a2qfnufwrtd39x2n50gp23h2qmdw6ns56d55j").try_into().unwrap()
        ];

        let wallet_states = build_default(actors).await.unwrap();
        let txs: Vec<Transaction> = wallet_states
            .into_iter()
            .filter_map(|state| state.registration_tx)
            .collect();

        let ledger = Ledger::new(Block0 {
            block: BlockBuilder::next_block(None, &txs),
            settings: Settings::default(),
        });

        let blockchain = ledger.blockchain();
        let last = blockchain.iter().last().unwrap();

        assert_eq!(last.transaction_bodies().len(), 4);
    }
}
