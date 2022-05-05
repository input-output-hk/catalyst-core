use catalyst_toolbox::snapshot::registration::{Delegations, VotingRegistration};
use chain_addr::Discrimination;
use jormungandr_lib::crypto::account::SigningKey;
use vitup::config::Block0Initial;

pub struct MainnetWallet {
    inner: thor::Wallet,
    reward_address: String,
    stake_public_key: String,
    stake: u64,
}

impl MainnetWallet {
    pub fn new(stake: u64) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            inner: thor::Wallet::new_account(&mut rng, Discrimination::Production),
            stake,
            reward_address: "0x".to_owned()
                + &SigningKey::generate_extended(&mut rng)
                    .identifier()
                    .to_hex(),
            stake_public_key: "0x".to_owned()
                + &SigningKey::generate_extended(&mut rng)
                    .identifier()
                    .to_hex(),
        }
    }

    pub fn reward_address(&self) -> String {
        self.reward_address.clone()
    }

    pub fn stake_public_key(&self) -> String {
        self.stake_public_key.clone()
    }

    pub fn catalyst_secret_key(&self) -> chain_crypto::SecretKey<chain_crypto::Ed25519Extended> {
        self.inner.secret_key()
    }

    pub fn as_voting_registration(&self) -> VotingRegistration {
        VotingRegistration {
            stake_public_key: self.stake_public_key(),
            voting_power: self.stake.into(),
            reward_address: self.reward_address(),
            delegations: Delegations::Legacy(self.inner.identifier().into()),
            voting_purpose: 0,
        }
    }

    pub fn as_initial_entry(&self) -> Block0Initial {
        Block0Initial::External {
            address: self.inner.address().to_string(),
            funds: self.stake,
            role: Default::default(),
        }
    }
}
