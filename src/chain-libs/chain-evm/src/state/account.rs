use crate::state::{storage::Storage, trie::Trie};

use primitive_types::{H160, U256};

pub type Nonce = U256;
pub type Balance = U256;

/// A represantation of an EVM account.
#[derive(Clone, Default)]
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

impl Account {
    pub fn is_empty(&self) -> bool {
        self.nonce == Nonce::zero() && self.balance == Balance::zero() && self.storage.is_empty()
    }
}

/// An address of an EVM account.
pub type AccountAddress = H160;

/// In-memory representation of all accounts.
pub type AccountTrie = Trie<AccountAddress, Account>;

impl AccountTrie {
    pub fn modify_account(
        &self,
        address: &AccountAddress,
        nonce: Nonce,
        balance: Balance,
        code: Option<Vec<u8>>,
        reset_storage: bool,
    ) -> Account {
        let account = if let Some(acct) = self.get(&address) {
            acct.clone()
        } else {
            Default::default()
        };
        let acct_storage = if reset_storage {
            Default::default()
        } else {
            account.storage
        };
        let code = if let Some(code) = code {
            code
        } else {
            account.code
        };

        Account {
            nonce,
            balance,
            storage: acct_storage,
            code,
        }
    }
}
