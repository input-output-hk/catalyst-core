use super::Actor;
use crate::wallet_state::MainnetWalletState;
use crate::CardanoWallet;
use jormungandr_lib::crypto::account::Identifier;

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
) -> Result<MainnetWalletState, Error> {
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
                    .find_map(|(name, key)| (rep == *name).then_some(key))
                {
                    Ok((identifier.clone(), *weight))
                } else {
                    Ok(Identifier::from_hex(rep)
                        .map(|identifier| (identifier, *weight))
                        .map_err(|_| Error::CannotGetVotingKey)?)
                }
            })
            .collect();

        let wallet = CardanoWallet::new(*ada);

        Ok(MainnetWalletState {
            rep: None,
            registration_tx: Some(wallet.generate_delegated_voting_registration(delegations?, 0)),
            stake: wallet.stake(),
            stake_address: wallet.reward_address().to_address(),
        })
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
