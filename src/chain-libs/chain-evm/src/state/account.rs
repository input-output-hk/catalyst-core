use crate::state::{storage::Storage, trie::Trie};
use crate::Address;

use primitive_types::U256;

pub type Nonce = U256;
pub type Balance = U256;

/// Smart-contract bytecode, such as the one compiled from Solidity code, for example.
pub type ByteCode = Vec<u8>;

/// A represantation of an EVM account.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Account {
    /// Account nonce. A number of value transfers from this account.
    pub nonce: Nonce,
    /// Account balance.
    pub balance: Balance,
    /// Account data storage.
    pub storage: Storage,
    /// EVM bytecode of this account.
    pub code: ByteCode,
}

impl Account {
    pub fn is_empty(&self) -> bool {
        self.nonce == Nonce::zero() && self.balance == Balance::zero() && self.storage.is_empty()
    }
}

/// In-memory representation of all accounts.
pub type AccountTrie = Trie<Address, Account>;

impl AccountTrie {
    /// Modify account
    ///
    /// If the element is not present, the closure F is apllied to the Default::default() value,
    /// otherwise the closure F is applied to the found element.
    /// If the closure returns None, then the key is deleted
    pub fn modify_account<F>(self, address: Address, f: F) -> Self
    where
        F: FnOnce(Account) -> Option<Account>,
    {
        let account = match self.get(&address) {
            Some(account) => account.clone(),
            None => Default::default(),
        };

        match f(account) {
            Some(account) => self.put(address, account),
            None => self.remove(&address),
        }
    }
}
