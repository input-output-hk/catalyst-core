//! # Bip44 derivation scheme
//!
//! based on the [BIP-0044] scheme.
//!
//! While nearly only the full bip44 address is indeed interesting, it is
//! valuable to keep the different intermediate steps as they can be reused
//! to define specific API objects.
//!
//! For example, for example a wallet with support to multiple coin types
//! will be interested to keep the `m/'44` path. For every account it is
//! interesting to keep the `m/'44/'<coin_type>/'<account>`.
//!
//! We have the 5 levels of Derivations: `m / purpose' / coin_type' / account' / change / address_index`
//!
//! # Examples
//!
//! basic usage:
//!
//! ```
//! # use chain_path_derivation::{Derivation, HardDerivation};
//! # const BITCOIN: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_0000));
//! # const ACCOUNT_01: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_0000));
//! #
//! use chain_path_derivation::bip44;
//!
//! let account = bip44::new().coin_type(BITCOIN).account(ACCOUNT_01);
//! assert_eq!(account.to_string(), "m/'44/'0/'0")
//! ```
//!
//! then it is possible to generate addresses from there:
//!
//! ```
//! # use chain_path_derivation::{Derivation, HardDerivation, SoftDerivation};
//! # const BITCOIN: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_0000));
//! # const ACCOUNT_01: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_0000));
//! #
//! # use chain_path_derivation::bip44;
//! #
//! # let account = bip44::new().coin_type(BITCOIN).account(ACCOUNT_01);
//! let change = account.external();
//! let first_address = change.address(SoftDerivation::min_value().wrapping_add(0));
//! let second_address = change.address(SoftDerivation::min_value().wrapping_add(1));
//! assert_eq!(first_address.to_string(), "m/'44/'0/'0/0/0");
//! assert_eq!(second_address.to_string(), "m/'44/'0/'0/0/1");
//! ```

use crate::{
    Derivation, DerivationPath, DerivationPathRange, HardDerivation, SoftDerivation,
    SoftDerivationRange,
};

/// scheme for the Bip44 chain path derivation
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bip44<P>(std::marker::PhantomData<P>);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Purpose;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoinType;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Account;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Change;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address;

const INDEX_PURPOSE: usize = 0;
const INDEX_COIN_TYPE: usize = 1;
const INDEX_ACCOUNT: usize = 2;
const INDEX_CHANGE: usize = 3;
const INDEX_ADDRESS: usize = 4;

/// the BIP44 purpose ('44). This is the first item of the derivation path
const PURPOSE: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_002C));

/// create a Bip44 chain path derivation
///
/// This derivation level is not really interesting, though it is interesting
/// to consider the following levels which can then be constructed via
/// each individual type.
///
/// See [module documentation] for more details
///
/// [module documentation]: ./index.html
#[inline]
pub fn new() -> DerivationPath<Bip44<Purpose>> {
    let mut dp = DerivationPath::new_empty();
    dp.push(PURPOSE.into());
    dp
}

impl DerivationPath<Bip44<Purpose>> {
    /// add the next derivation level for the Bip44 chain path derivation.
    ///
    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn coin_type(&self, coin_type: HardDerivation) -> DerivationPath<Bip44<CoinType>> {
        let mut ct = self.clone();
        ct.push(coin_type.into());
        ct.coerce_unchecked()
    }
}

impl DerivationPath<Bip44<CoinType>> {
    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn account(&self, account: HardDerivation) -> DerivationPath<Bip44<Account>> {
        let mut a = self.clone();
        a.push(account.into());
        a.coerce_unchecked()
    }
}

impl DerivationPath<Bip44<Account>> {
    const EXTERNAL: SoftDerivation = SoftDerivation::new_unchecked(Derivation::new(0));
    const INTERNAL: SoftDerivation = SoftDerivation::new_unchecked(Derivation::new(1));

    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn change(&self, change: SoftDerivation) -> DerivationPath<Bip44<Change>> {
        let mut c = self.clone();
        c.push(change.into());
        c.coerce_unchecked()
    }

    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn external(&self) -> DerivationPath<Bip44<Change>> {
        self.change(Self::EXTERNAL)
    }

    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn internal(&self) -> DerivationPath<Bip44<Change>> {
        self.change(Self::INTERNAL)
    }
}

impl DerivationPath<Bip44<Change>> {
    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn address(&self, address: SoftDerivation) -> DerivationPath<Bip44<Address>> {
        let mut a = self.clone();
        a.push(address.into());
        a.coerce_unchecked()
    }

    /// build a range of addresses
    ///
    /// # panics
    ///
    /// This function will panic is the range is out of bounds for a valid
    /// address (`SoftDerivation`).
    ///
    /// # Examples
    ///
    /// Generate the first 20 chain path derivation addresses, from 0 to 19 (inclusive):
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, HardDerivation, SoftDerivation};
    /// # const BITCOIN: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_0000));
    /// # const ACCOUNT_01: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_0000));
    /// #
    /// # use chain_path_derivation::bip44;
    /// #
    /// let account = bip44::new().coin_type(BITCOIN).account(ACCOUNT_01);
    /// let change = account.external();
    /// let end = SoftDerivation::min_value().saturating_add(20);
    ///
    /// let addresses = change.addresses((..end)).collect::<Vec<_>>();
    ///
    /// assert_eq!(addresses[0].to_string(), "m/'44/'0/'0/0/0");
    /// assert_eq!(addresses[1].to_string(), "m/'44/'0/'0/0/1");
    /// // ..
    /// assert_eq!(addresses[19].to_string(), "m/'44/'0/'0/0/19");
    /// ```
    pub fn addresses<R, T>(
        &self,
        range: R,
    ) -> DerivationPathRange<DerivationPath<Bip44<Address>>, SoftDerivationRange, SoftDerivation>
    where
        R: std::ops::RangeBounds<T>,
        T: std::convert::TryInto<SoftDerivation> + Copy,
        <T as std::convert::TryInto<SoftDerivation>>::Error: std::error::Error,
    {
        let range = SoftDerivationRange::new(range);

        self.clone().coerce_unchecked().sub_range(range)
    }
}

impl DerivationPath<Bip44<Address>> {
    #[inline]
    fn get_unchecked(&self, index: usize) -> Derivation {
        if let Some(v) = self.get(index) {
            v
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    #[inline]
    pub fn purpose(&self) -> Derivation {
        self.get_unchecked(INDEX_PURPOSE)
    }

    #[inline]
    pub fn coin_type(&self) -> Derivation {
        self.get_unchecked(INDEX_COIN_TYPE)
    }

    #[inline]
    pub fn account(&self) -> Derivation {
        self.get_unchecked(INDEX_ACCOUNT)
    }

    #[inline]
    pub fn change(&self) -> Derivation {
        self.get_unchecked(INDEX_CHANGE)
    }

    #[inline]
    pub fn address(&self) -> Derivation {
        self.get_unchecked(INDEX_ADDRESS)
    }
}
