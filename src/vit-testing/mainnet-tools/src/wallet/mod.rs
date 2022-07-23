mod sender;

use bech32::ToBase32;
use catalyst_toolbox::snapshot::registration::{Delegations, VotingRegistration};
use chain_addr::Discrimination;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::crypto::account::SigningKey;
use jormungandr_lib::interfaces::Address;
use sender::RegistrationSender;

pub struct MainnetWallet {
    catalyst: thor::Wallet,
    reward_address: String,
    stake_public_key: String,
    stake: u64,
}

impl MainnetWallet {
    pub fn new(stake: u64) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            catalyst: thor::Wallet::new_account(&mut rng, Discrimination::Production),
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

    pub fn reward_address_as_bech32(&self) -> String {
        let bytes = hex::decode(self.reward_address.clone().trim_start_matches("0x")).unwrap();
        bech32::encode("stake", &bytes.to_base32(), bech32::Variant::Bech32).unwrap()
    }

    pub fn stake_public_key(&self) -> String {
        self.stake_public_key.clone()
    }

    pub fn catalyst_secret_key(&self) -> chain_crypto::SecretKey<chain_crypto::Ed25519Extended> {
        self.catalyst.secret_key()
    }

    pub fn catalyst_public_key(&self) -> Identifier {
        self.catalyst.secret_key().to_public().into()
    }

    pub fn catalyst_address(&self) -> Address {
        self.catalyst.address()
    }

    pub fn send_voting_registration(&self, delegations: Delegations) -> RegistrationSender {
        RegistrationSender::new(VotingRegistration {
            stake_public_key: self.stake_public_key(),
            voting_power: self.stake.into(),
            reward_address: self.reward_address(),
            delegations,
            voting_purpose: 0,
        })
    }

    pub fn send_direct_voting_registration(&self) -> RegistrationSender {
        self.send_voting_registration(Delegations::Legacy(self.catalyst.identifier().into()))
    }

    pub fn stake(&self) -> u64 {
        self.stake
    }
}
