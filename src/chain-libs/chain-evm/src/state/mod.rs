/*!
This module contains a simple representation of all EVM-related data based on
immutable data structures. It is designed to be used with the ledger
implementation from `chain-impl-mockchain` in Jormungandr multiverse of ledgers.
*/

mod account;
mod logs;
mod storage;
mod trie;

pub use account::{Account, AccountState, AccountTrie, Balance, ByteCode, Nonce};
use evm::ExitError;
pub use logs::LogsState;
use std::borrow::Cow;
pub use storage::{Key, Storage, Value};
pub use trie::Trie;

/// Definition for state-related errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("EVM values cannot exceed 64 significant bits")]
    ValueOverflow,
}

impl From<Error> for crate::machine::Error {
    fn from(other: Error) -> Self {
        Self::TransactionError(ExitError::Other(Cow::from(String::from(other))))
    }
}

impl From<Error> for String {
    fn from(other: Error) -> Self {
        format!("{:?}", other)
    }
}
