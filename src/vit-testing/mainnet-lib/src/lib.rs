//! Rust implementation of Cardano various tools
//!
//! [`DbSyncInstance`] - mock of db sync based on json file rather than postgres db
//! Network - mock of cardano network in memory
//! Wallet - api implementation of cardano wallet which is capable of sending and signing registration
//! transactions

#![forbid(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_bool,
    clippy::bool_assert_comparison,
    clippy::derive_partial_eq_without_eq
)]

mod db_sync;
mod network;
mod wallet;
mod blockfrost;
mod cardano_node;

pub use exports::*;
mod exports {
    pub use crate::db_sync::{BlockDateFromCardanoAbsoluteSlotNo, InMemoryDbSync, SharedInMemoryDbSync, Error as DbSyncError};
    pub use crate::network::{
        MainnetNetworkBuilder, MainnetWalletState, MainnetWalletStateBuilder,
    };
    pub use crate::wallet::{
        CardanoWallet, GeneralTransactionMetadataInfo, JsonConversionError, METADATUM_1, METADATUM_2, METADATUM_3, METADATUM_4, REGISTRATION_METADATA_IDX,
        REGISTRATION_METADATA_LABEL, REGISTRATION_METADATA_SIGNATURE_LABEL, REGISTRATION_SIGNATURE_METADATA_IDX, RegistrationTransactionBuilder
    };
    pub use crate::blockfrost::{CatalystBlockFrostApi, Error as CatalystBlockFrostApiError};

    pub use crate::cardano_node::{InMemoryNode, Ledger, Settings};
    pub use crate::cardano_node::{Block0,BlockBuilder,TransactionBuilder};
}
