/*!
This module contains a simple representation of all EVM-related data based on
immutable data structures. It is designed to be used with the ledger
implementation from `chain-impl-mockchain` in Jormungandr multiverse of ledgers.
*/

mod account;
mod storage;
mod trie;

pub use account::{Account, AccountTrie, Balance, ByteCode, Nonce};
pub use storage::{Key, Storage, Value};
pub use trie::Trie;
