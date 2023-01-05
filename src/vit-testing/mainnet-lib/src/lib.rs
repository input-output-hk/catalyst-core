//! Rust implementation of Cardano various tools
//!
//! [`InMemoryDbSync`] - mock of db sync based on json file rather than postgres db
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

mod blockfrost;
mod cardano_node;
mod db_sync;
mod network;
mod wallet;

/// Const defining caradno mainnet slot per epoch setting
pub const CARDANO_MAINNET_SLOTS_PER_EPOCH: u64 = 43200;

pub use exports::*;
mod exports {
    pub use crate::blockfrost::{CatalystBlockFrostApi, Error as CatalystBlockFrostApiError};
    pub use crate::db_sync::{
        BlockDateFromCardanoAbsoluteSlotNo, Error as DbSyncError, InMemoryDbSync,
        SharedInMemoryDbSync,
    };
    pub use crate::wallet::{
        CardanoWallet, GeneralTransactionMetadataInfo, JsonConversionError,
        RegistrationTransactionBuilder, METADATUM_1, METADATUM_2, METADATUM_3, METADATUM_4,
        REGISTRATION_METADATA_IDX, REGISTRATION_METADATA_LABEL,
        REGISTRATION_METADATA_SIGNATURE_LABEL, REGISTRATION_SIGNATURE_METADATA_IDX,
    };

    pub use crate::cardano_node::{Block0, BlockBuilder, TransactionBuilder};
    pub use crate::cardano_node::{InMemoryNode, Ledger, Settings};
    pub use crate::network::{
        wallet_state, Initials, MainnetNetworkBuilder, Parameters as SnapshotParameters,
    };
}
