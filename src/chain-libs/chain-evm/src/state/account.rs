use crate::{
    state::{storage::Storage, trie::Trie, Error},
    Address,
};
use ethereum_types::U256;

pub type Nonce = U256;

/// Ethereum account balance which uses the least 64 significant bits of the `U256` type.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd)]
pub struct Balance(u64);

impl std::fmt::Display for Balance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<U256> for Balance {
    type Error = Error;
    fn try_from(other: U256) -> Result<Self, Self::Error> {
        match other {
            U256([val, 0, 0, 0]) => Ok(Balance(val)),
            _ => Err(Error::ValueOverflow),
        }
    }
}

impl From<u64> for Balance {
    fn from(other: u64) -> Self {
        Balance(other)
    }
}

impl From<Balance> for u64 {
    fn from(other: Balance) -> Self {
        other.0
    }
}

impl From<Balance> for U256 {
    fn from(other: Balance) -> U256 {
        other.0.into()
    }
}

impl Balance {
    /// Zero (additive identity) of this type.
    pub fn zero() -> Self {
        Balance(0)
    }
    /// Returns `Some(balance)` or `None` if overflow occurred.
    pub fn checked_add(self, other: Balance) -> Option<Balance> {
        self.0.checked_add(other.0).map(Self)
    }
    /// Returns `Some(balance)` or `None` if overflow occurred.
    pub fn checked_sub(self, other: Balance) -> Option<Balance> {
        self.0.checked_sub(other.0).map(Self)
    }
}

/// Smart-contract bytecode, such as the one compiled from Solidity code, for example.
pub type ByteCode = Vec<u8>;

/// A represantation of an EVM account.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Account {
    /// Account balance.
    pub balance: Balance,
    /// Account state.
    pub state: AccountState,
}

/// A representation of an EVM account state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountState {
    /// Account data storage.
    pub storage: Storage,
    /// EVM bytecode of this account.
    pub code: ByteCode,
    /// Account nonce. A number of value transfers from this account.
    pub nonce: Nonce,
}

impl Account {
    pub fn is_empty(&self) -> bool {
        self.state.nonce.is_zero()
            && self.balance == Balance::zero()
            && self.state.storage.is_empty()
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

#[cfg(test)]
mod test {
    use super::*;

    const MAX_SIZE: u64 = u64::MAX;

    #[test]
    fn account_balance_u256_zero() {
        assert_eq!(Balance::zero(), Balance(0));
    }

    #[test]
    fn account_balance_u256_checked_add() {
        let val = 100u64;
        assert_eq!(
            Balance::from(val).checked_add(U256::from(0u64).try_into().unwrap()),
            Some(Balance(val))
        );
        assert_eq!(
            Balance(MAX_SIZE).checked_add(U256::from(1u64).try_into().unwrap()),
            None
        );
    }

    #[test]
    fn account_balance_u256_checked_sub() {
        let val = 100u64;
        assert_eq!(
            Balance::from(val).checked_sub(U256::from(0u64).try_into().unwrap()),
            Some(Balance(val))
        );
        assert_eq!(
            Balance::from(0u64).checked_sub(U256::from(1u64).try_into().unwrap()),
            None
        );
    }

    #[test]
    fn account_balance_u256_can_never_use_more_than_64_bits() {
        // convert from u64
        assert_eq!(Balance::from(MAX_SIZE), Balance(MAX_SIZE));
        // try to convert from U256
        assert!(Balance::try_from(U256::from(MAX_SIZE)).is_ok());
        assert!(Balance::try_from(U256::from(MAX_SIZE) + U256::from(1_u64)).is_err());

        // Anything larger than the least significant 64 bits
        // returns error
        assert!(Balance::try_from(U256([0, 1, 0, 0])).is_err());
        assert!(Balance::try_from(U256([0, 0, 1, 0])).is_err());
        assert!(Balance::try_from(U256([0, 0, 0, 1])).is_err());
    }
}
