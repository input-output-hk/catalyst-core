/*!
This module contains a simple representation of all EVM-related data based on
immutable data structures. It is designed to be used with the ledger
implementation from `chain-impl-mockchain` in Jormungandr multiverse of ledgers.
*/

mod account;
mod logs;
mod storage;
mod trie;

pub use account::{Account, AccountTrie, Balance, ByteCode, Nonce};
pub use logs::LogsState;
pub use storage::{Key, Storage, Value};
pub use trie::Trie;

/// Definition for state-related errors.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("account balance values cannot exceed 64 significant bits")]
    BalanceOverflow,
}

impl From<Error> for crate::machine::Error {
    fn from(other: Error) -> Self {
        Self::StateError(other)
    }
}

impl From<Error> for String {
    fn from(other: Error) -> Self {
        format!("{:?}", other)
    }
}
