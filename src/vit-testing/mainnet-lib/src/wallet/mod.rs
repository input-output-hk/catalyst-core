mod registration;

pub use crate::wallet::registration::{
    GeneralTransactionMetadataInfo, JsonConversionError, RegistrationTransactionBuilder,
    METADATUM_1, METADATUM_2, METADATUM_3, METADATUM_4, REGISTRATION_METADATA_IDX,
    REGISTRATION_METADATA_LABEL, REGISTRATION_METADATA_SIGNATURE_LABEL,
    REGISTRATION_SIGNATURE_METADATA_IDX,
};
use cardano_serialization_lib::{
    BaseAddress, NetworkInfo, PrivateKey, PublicKey, RewardAddress, StakeCredential,
};
use cardano_serialization_lib::Transaction;
use chain_addr::Discrimination;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::Address;
use rand::{CryptoRng, RngCore};
use snapshot_lib::registration::Delegations;

/// Represents Cardano mainnet wallet which is able to generate registration transaction metadata
pub struct CardanoWallet {
    catalyst: thor::Wallet,
    stake_key: PrivateKey,
    payment_key: PrivateKey,
    network: NetworkInfo,
    stake: u64,
}

impl CardanoWallet {
    /// Creates new wallet with given ada. Currently wallet is purely used for testing purposes,
    /// therefore we treat stake as arbitrary number not connected to any blockchain state.
    /// # Panics
    ///
    /// Panics on key generation error
    #[must_use]
    pub fn new(stake: u64) -> Self {
        let rng = rand::thread_rng();
        Self::new_with_rng(stake, rng)
    }

    /// Creates new wallet with given ada and rng. Currently wallet is purely used for testing purposes,
    /// therefore we treat stake as arbitrary number not connected to any blockchain state.
    /// # Panics
    ///
    /// Panics on key generation error
    #[must_use]
    pub fn new_with_rng<T: RngCore + CryptoRng>(stake: u64, mut rng: T) -> Self {
        Self {
            catalyst: thor::Wallet::new_account(&mut rng, Discrimination::Production),
            stake_key: PrivateKey::generate_ed25519extended().unwrap(),
            payment_key: PrivateKey::generate_ed25519extended().unwrap(),
            network: NetworkInfo::mainnet(),
            stake,
        }
    }

    /// Rewards address based on stake public key
    #[must_use]
    pub fn reward_address(&self) -> RewardAddress {
        RewardAddress::new(self.network.network_id(), &self.stake_credential())
    }

    /// Stake address based on stake public key
    #[must_use]
    pub fn stake_credential(&self) -> StakeCredential {
        StakeCredential::from_keyhash(&self.stake_key.to_public().hash())
    }

    /// Payment address based on stake public key
    #[must_use]
    pub fn payment_credential(&self) -> StakeCredential {
        StakeCredential::from_keyhash(&self.payment_key.to_public().hash())
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

    /// Mainnet address
    #[must_use]
    pub fn address(&self) -> BaseAddress {
        BaseAddress::new(
            self.network.network_id(),
            &self.payment_credential(),
            &self.stake_credential(),
        )
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
    ) -> Transaction {
        RegistrationTransactionBuilder::new(self)
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
    ) -> Transaction {
        self.generate_voting_registration(Delegations::New(delegations), slot_no)
    }

    /// Creates direct (a.k.a self) voting registration metadata according to [Cip-36](https://cips.cardano.org/cips/cip36/) on given absolut slot number.
    #[must_use]
    pub fn generate_direct_voting_registration(&self, slot_no: u64) -> Transaction {
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
