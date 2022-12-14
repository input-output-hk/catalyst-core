pub mod arbitrary;
pub mod blockfrost;
pub mod definition;

pub use crate::mock::builder::arbitrary::{generate_arbitrary_delegator, Error as ArbitraryError};
pub use crate::mock::builder::definition::{delegator, registration, representative, Actor};
use crate::mock::config::Providers;
use crate::mock::Error;
use cardano_serialization_lib::Transaction;
use jormungandr_lib::crypto::account::Identifier;
use mainnet_lib::CardanoWallet;
use mainnet_lib::{Block0, BlockBuilder};

pub async fn build_block0(
    actors: Vec<Actor>,
    _provider_config: Providers,
) -> Result<Block0, Error> {
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
                    Identifier::from_hex(voting_key)
                        .map_err(|e| Error::PublicKeyFromStr(e.to_string()))?,
                ));
            }
            _ => {
                return Err(Error::Internal(
                    "collection of external reps should be already filtered out".to_string(),
                ))
            }
        }
    }

    let delegators: Result<Vec<Transaction>, Error> = generated_delegation_def
        .iter()
        .map(|actor| Ok(generate_arbitrary_delegator(actor, &voting_keys)?))
        .collect();
    let mut delegators: Vec<Transaction> = delegators?;

    delegators.extend(blockfrost::download_registrations_from_network(&actors).await?);
    Ok(Block0 {
        block: BlockBuilder::next_block(None, &delegators),
        settings: Default::default(),
    })
}

#[cfg(test)]
mod tests {
    use crate::mock::builder::{build_block0, delegator, registration, representative, Providers};
    use mainnet_lib::Ledger;

    #[tokio::test]
    pub async fn test() {
        let actors = vec![
            representative("alice").with_ada(1000).try_into().unwrap(),
            representative("bob").with_key("e45f2426ffdcf4e819ca38c15adb396d9c1519ffa7239f11ae8cd79f15bc38b5").try_into().unwrap(),
            delegator("clarice").with_registration(registration().at_slot(1).with_targets(vec![("bob",1)]).try_into().unwrap()).with_ada(100).try_into().unwrap(),
            delegator("david").with_address("addr_test1qqpjfknn6azgwjs0u4x7cjmng8x7m2uve0eyqk2vafa9ghc68ap070sg8any3a2qfnufwrtd39x2n50gp23h2qmdw6ns56d55j").try_into().unwrap()
        ];

        let block0 = build_block0(actors, Providers::BlockFrost).await.unwrap();
        let ledger = Ledger::new(block0);

        let blockchain = ledger.blockchain();
        let last = blockchain.iter().last().unwrap();

        assert_eq!(last.transaction_bodies().len(), 4);
    }
}
