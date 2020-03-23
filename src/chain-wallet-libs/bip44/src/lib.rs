//! BIP44 addressing
//!
//! provides all the logic to create safe sequential addresses
//! using BIP44 specification.
//!

use std::{fmt, result};
use thiserror::Error;

/// the BIP44 derivation path has a specific length
pub const BIP44_PATH_LENGTH: usize = 5;

/// the BIP44 derivation path has a specific purpose
pub const BIP44_PURPOSE: u32 = 0x8000_002C;

/// the BIP44 coin type is set, by default, to cardano ada.
pub const BIP44_COIN_TYPE: u32 = 0x8000_0717;

/// the soft derivation is upper bounded
pub const BIP44_SOFT_UPPER_BOUND: u32 = 0x8000_0000;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Path(pub Vec<u32>);

/// Error relating to `bip44`'s `Addressing` operations
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Error)]
pub enum Error {
    /// this means the given `Path` has an incompatible length
    /// for bip44 derivation. See `BIP44_PATH_LENGTH` and `Addressing::from_path`.
    #[error("Invalid length, expecting 5 but received {0}")]
    InvalidLength(usize),

    /// this means the given `Path` has an incompatible purpose
    /// for bip44 derivation. See `BIP44_PURPOSE` and `Addressing::from_path`.
    #[error("Invalid purpose, expecting 0x8000002C but received 0x{0:x}")]
    InvalidPurpose(u32),

    /// this means the given `Path` has an incompatible coin type
    /// for bip44 derivation. See `BIP44_COIN_TYPE` and `Addressing::from_path`.
    #[error("Invalid type, expecting 0x80000717 but received 0x{0:x}")]
    InvalidType(u32),

    /// this means the given `Path` has an incompatible account
    /// for bip44 derivation. That it is out of bound. Indeed
    /// the account derivation is expected to be a hard derivation.
    #[error("Account out of bound, should have a hard derivation but received 0x{0:x}")]
    AccountOutOfBound(u32),

    /// this means the given `Path` has an incompatible change
    /// for bip44 derivation. That it is out of bound. Indeed
    /// the change derivation is expected to be a soft derivation.
    #[error("Change out of bound, should have a soft derivation but received 0x{0:x}")]
    ChangeOutOfBound(u32),

    /// this means the given `Path` has an incompatible index
    /// for bip44 derivation. That it is out of bound. Indeed
    /// the index derivation is expected to be a soft derivation.
    #[error("Index out of bound, should have a soft derivation but received 0x{0:x}")]
    IndexOutOfBound(u32),
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Account(u32);
impl Account {
    pub fn new(account: u32) -> Result<Self> {
        if account >= BIP44_SOFT_UPPER_BOUND {
            return Err(Error::AccountOutOfBound(account));
        }
        Ok(Account(account))
    }

    pub fn get_account_number(self) -> u32 {
        self.0
    }
    pub fn get_scheme_value(self) -> u32 {
        self.0 | BIP44_SOFT_UPPER_BOUND
    }

    pub fn change(self, typ: AddrType) -> Result<Change> {
        match typ {
            AddrType::Internal => self.internal(),
            AddrType::External => self.external(),
        }
    }

    pub fn internal(self) -> Result<Change> {
        Change::new(self, 1)
    }
    pub fn external(self) -> Result<Change> {
        Change::new(self, 0)
    }
}
impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(u32);
impl Index {
    pub fn new(index: u32) -> Result<Self> {
        if index >= BIP44_SOFT_UPPER_BOUND {
            return Err(Error::IndexOutOfBound(index));
        }
        Ok(Index(index))
    }
    pub fn get_scheme_value(self) -> u32 {
        self.0
    }
    pub fn incr(self, i: u32) -> Result<Self> {
        if i >= BIP44_SOFT_UPPER_BOUND {
            return Err(Error::IndexOutOfBound(i));
        }
        let r = self.0 + i;
        if r >= BIP44_SOFT_UPPER_BOUND {
            return Err(Error::IndexOutOfBound(r));
        }
        Ok(Index(r))
    }

    pub fn decr(self, i: u32) -> Result<Self> {
        if self.0 < i {
            return Err(Error::IndexOutOfBound(0));
        }
        let r = self.0 - i;
        Ok(Index(r))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Change {
    account: Account,
    change: u32,
}
impl Change {
    pub fn new(account: Account, change: u32) -> Result<Self> {
        if change >= BIP44_SOFT_UPPER_BOUND {
            return Err(Error::ChangeOutOfBound(change));
        }
        Ok(Change { account, change })
    }
    pub fn get_scheme_value(self) -> u32 {
        self.change
    }

    pub fn index(self, index: u32) -> Result<Addressing> {
        Addressing::new_from_change(self, index)
    }
}

/// Bip44 address derivation
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Addressing {
    pub account: Account,
    pub change: u32,
    pub index: Index,
}
impl fmt::Display for Addressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.account.0, self.change, self.index.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddrType {
    Internal,
    External,
}

impl Addressing {
    /// create a new `Addressing` for the given account, `AddrType`
    /// and address index.
    pub fn new(account: u32, typ: AddrType, index: u32) -> Result<Self> {
        let change = match typ {
            AddrType::Internal => 1,
            AddrType::External => 0,
        };
        Ok(Addressing {
            account: Account::new(account)?,
            change,
            index: Index::new(index)?,
        })
    }

    fn new_from_change(change: Change, index: u32) -> Result<Self> {
        Ok(Addressing {
            account: change.account,
            change: change.change,
            index: Index::new(index)?,
        })
    }

    /// return a path ready for derivation
    pub fn to_path(&self) -> Path {
        Path(vec![
            BIP44_PURPOSE,
            BIP44_COIN_TYPE,
            self.account.get_scheme_value(),
            self.change,
            self.index.get_scheme_value(),
        ])
    }

    pub fn address_type(&self) -> AddrType {
        if self.change == 0 {
            AddrType::External
        } else {
            AddrType::Internal
        }
    }

    pub fn from_path(path: Path) -> Result<Self> {
        let len = path.0.len();
        if path.0.len() != BIP44_PATH_LENGTH {
            return Err(Error::InvalidLength(len));
        }

        let purpose = path.0[0];
        if purpose != BIP44_PURPOSE {
            return Err(Error::InvalidPurpose(purpose));
        }
        let coin_type = path.0[1];
        if coin_type != BIP44_COIN_TYPE {
            return Err(Error::InvalidType(coin_type));
        }
        let account = path.0[2];
        let change = path.0[3];
        let index = path.0[4];

        Account::new(account)
            .and_then(|account| Change::new(account, change))
            .and_then(|change| Addressing::new_from_change(change, index))
    }

    /// try to generate a new `Addressing` starting from the given
    /// `Addressing`'s index incremented by the given parameter;
    pub fn incr(&self, incr: u32) -> Result<Self> {
        let mut addr = *self;
        addr.index = addr.index.incr(incr)?;
        Ok(addr)
    }

    /// generate a sequence of Addressing from the given
    /// addressing as starting point up to the `chunk_size`.
    ///
    /// the function will return as soon as `chunk_size` is reached
    /// or at the first `Error::IndexOutOfBound`.
    ///
    pub fn next_chunks(&self, chunk_size: usize) -> Result<Vec<Self>> {
        let mut v = Vec::with_capacity(chunk_size);
        for i in 0..chunk_size {
            match self.incr(i as u32) {
                Err(Error::IndexOutOfBound(_)) => break,
                Err(err) => return Err(err),
                Ok(r) => v.push(r),
            }
        }
        Ok(v)
    }
}
