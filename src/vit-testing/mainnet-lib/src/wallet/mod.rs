mod registration;

use crate::wallet::registration::RegistrationBuilder;
use cardano_serialization_lib::address::{NetworkInfo, RewardAddress, StakeCredential};
use cardano_serialization_lib::crypto::{PrivateKey, PublicKey};
use cardano_serialization_lib::metadata::GeneralTransactionMetadata;
use chain_addr::Discrimination;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Address;
use snapshot_lib::registration::Delegations;

/// Represents Cardano mainnet wallet which generate registration transaction metadata
pub struct MainnetWallet {
    catalyst: thor::Wallet,
    stake_key: PrivateKey,
    payment_key: PrivateKey,
    stake: u64,
}

impl MainnetWallet {
    /// Creates new wallet with given ada. Currently wallet is purely used for testing purposes,
    /// therefore we treat stake as arbitrary number not connected to any blockchain state.
    /// # Panics
    ///
    /// Panics on key generation error
    #[must_use]
    pub fn new(stake: u64) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            catalyst: thor::Wallet::new_account(&mut rng, Discrimination::Production),
            stake_key: PrivateKey::generate_ed25519extended().unwrap(),
            payment_key: PrivateKey::generate_ed25519extended().unwrap(),
            stake,
        }
    }

    /// Rewards address based on stake public key
    #[must_use]
    pub fn reward_address(&self) -> RewardAddress {
        let staking_key = StakeCredential::from_keyhash(&self.stake_key.to_public().hash());
        RewardAddress::new(NetworkInfo::mainnet().network_id(), &staking_key)
    }

    /// Cardano stake public key
    #[must_use]
    pub fn stake_public_key(&self) -> PublicKey {
        self.stake_key.to_public()
    }

    /// Catalyst secret key
    #[must_use]
    pub fn catalyst_secret_key(&self) -> chain_crypto::SecretKey<chain_crypto::Ed25519Extended> {
        self.catalyst.secret_key()
    }

    /// Catalyst public key
    #[must_use]
    pub fn catalyst_public_key(&self) -> Identifier {
        self.catalyst.secret_key().to_public().into()
    }

    /// Catalyst address
    #[must_use]
    pub fn catalyst_address(&self) -> Address {
        self.catalyst.address()
    }

    /// Creates voting registration metadata according to [Cip-36](https://cips.cardano.org/cips/cip36/) on given absolut slot number
    /// and based on [`Delegations`] object which can be either legacy or delegation.
    #[must_use]
    pub fn generate_voting_registration(
        &self,
        delegations: Delegations,
        slot_no: u64,
    ) -> GeneralTransactionMetadata {
        RegistrationBuilder::new(self)
            .to(delegations)
            .on(slot_no)
            .build()
    }

    /// Creates delegated voting registration metadata according to [Cip-36](https://cips.cardano.org/cips/cip36/) on given absolut slot number.
    #[must_use]
    pub fn generate_delegated_voting_registration(
        &self,
        delegations: Vec<(Identifier, u32)>,
        slot_no: u64,
    ) -> GeneralTransactionMetadata {
        self.generate_voting_registration(Delegations::New(delegations), slot_no)
    }

    /// Creates direct (a.k.a self) voting registration metadata according to [Cip-36](https://cips.cardano.org/cips/cip36/) on given absolut slot number.
    #[must_use]
    pub fn generate_direct_voting_registration(&self, slot_no: u64) -> GeneralTransactionMetadata {
        self.generate_voting_registration(
            Delegations::Legacy(self.catalyst.identifier().into()),
            slot_no,
        )
    }

    /// current amount of ada
    #[must_use]
    pub fn stake(&self) -> u64 {
        self.stake
    }

    /// private payment key
    #[must_use]
    pub fn payment_key(&self) -> &PrivateKey {
        &self.payment_key
    }

    /// private stake key
    #[must_use]
    pub fn stake_key(&self) -> &PrivateKey {
        &self.stake_key
    }
}
