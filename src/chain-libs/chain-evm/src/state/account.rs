use crate::state::{storage::Storage, trie::Trie};

use primitive_types::{H160, U256};

pub type Nonce = U256;
pub type Balance = U256;

/// A represantation of an EVM account.
#[derive(Clone)]
pub struct Account {
    /// Account nonce. A number of value transfers from this account.
    pub nonce: Nonce,
    /// Account balance.
    pub balance: Balance,
    /// Account data storage.
    pub storage: Storage,
    /// EVM bytecode of this account.
    pub code: Vec<u8>,
}

/// An address of an EVM account.
pub type AccountAddress = H160;

/// In-memory representation of all accounts.
pub type AccountTrie = Trie<AccountAddress, Account>;
