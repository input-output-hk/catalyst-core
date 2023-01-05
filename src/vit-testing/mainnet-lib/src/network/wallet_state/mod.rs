mod builder;
mod template;

pub use builder::MainnetWalletStateBuilder;
use std::fmt::{Debug, Formatter};
pub use template::{build, build_default, Actor, Error as TemplateError};

use cardano_serialization_lib::address::Address;
use cardano_serialization_lib::Transaction;
use jormungandr_lib::crypto::account::Identifier;

/// Represents wallet candidate for registration. Defines wallet role (delegator/direct-voter/representative)
pub struct MainnetWalletState {
    pub(crate) rep: Option<Identifier>,
    pub(crate) registration_tx: Option<Transaction>,
    pub(crate) stake: u64,
    pub(crate) stake_address: Address,
}

impl Debug for MainnetWalletState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("MainnetWalletState");
        fmt.field("rep", &self.rep);
        fmt.field("stake", &self.stake);
        fmt.field("stake_address", &self.stake_address.to_hex());
        fmt.field(
            "metadata",
            &self
                .registration_tx
                .as_ref()
                .map(Transaction::auxiliary_data),
        );
        fmt.finish()
    }
}

impl MainnetWalletState {
    /// get representative information
    #[must_use]
    pub fn rep(&self) -> &Option<Identifier> {
        &self.rep
    }
    /// get registration metadata
    #[must_use]
    pub fn registration(&self) -> &Option<Transaction> {
        &self.registration_tx
    }
    /// get wallet stake
    #[must_use]
    pub fn stake(&self) -> u64 {
        self.stake
    }
    /// get stake address for wallet
    #[must_use]
    pub fn stake_address(&self) -> &Address {
        &self.stake_address
    }
}
