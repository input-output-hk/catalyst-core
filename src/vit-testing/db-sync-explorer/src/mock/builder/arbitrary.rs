use super::Actor;
use cardano_serialization_lib::Transaction;
use jormungandr_lib::crypto::account::Identifier;
use mainnet_lib::CardanoWallet;

pub fn generate_arbitrary_representative(actor_def: &Actor) -> (String, CardanoWallet) {
    if let Actor::GeneratedRep { name, ada } = actor_def {
        (name.clone(), CardanoWallet::new(*ada))
    } else {
        panic!("internal error: expected generated rep for");
    }
}

pub fn generate_arbitrary_delegator(
    actor_def: &Actor,
    voting_keys: &[(&String, Identifier)],
) -> Result<Transaction, Error> {
    if let Actor::GeneratedDelegator {
        registration, ada, ..
    } = actor_def
    {
        let delegations: Result<Vec<_>, Error> = registration
            .target
            .iter()
            .map(|(rep, weight)| {
                if let Some(identifier) = voting_keys
                    .iter()
                    .filter_map(|(name, key)| (rep == *name).then_some(key))
                    .next()
                {
                    Ok((identifier.clone(), *weight))
                } else {
                    Ok(Identifier::from_hex(rep)
                        .map(|identifier| (identifier, *weight))
                        .map_err(|_| Error::CannotGetVotingKey)?)
                }
            })
            .collect();
        Ok(CardanoWallet::new(*ada).generate_delegated_voting_registration(delegations?, 0u64))
    } else {
        Err(Error::Internal("expected generated delegator".to_string()))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(String),
    #[error("cannot convert voting key")]
    CannotGetVotingKey,
}
