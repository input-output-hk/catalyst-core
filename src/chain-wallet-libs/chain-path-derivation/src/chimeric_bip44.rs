//! # cardano shelley HD Wallet derivation scheme
//!
//! based on the [BIP-0044] scheme but with different purpose and a new kind of
//! `change` to allow for account stake key.
//!
//! We have the 5 levels of Derivations: `m / purpose' / coin_type' / account' / type / address_index`
//!
//! see https://input-output-hk.github.io/adrestia/docs/key-concepts/hierarchical-deterministic-wallets/
//!

use crate::{
    bip44::Bip44, AnyScheme, Derivation, DerivationPath, DerivationPathRange, HardDerivation,
    ParseDerivationPathError, SoftDerivation, SoftDerivationRange,
};
use std::str::{self, FromStr};

pub use crate::bip44::{new, Root};

/// scheme for the ChimericBip44 chain path derivation
///
pub type ChimericBip44<A> = Bip44<A>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Purpose;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoinType;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Account;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Type;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address;

const INDEX_PURPOSE: usize = 0;
const INDEX_COIN_TYPE: usize = 1;
const INDEX_ACCOUNT: usize = 2;
const INDEX_TYPE: usize = 3;
const INDEX_ADDRESS: usize = 4;

/// the Chimeric BIP44 purpose ('1852). This is the first item of the derivation path
///
const PURPOSE: HardDerivation = HardDerivation::new_unchecked(Derivation::new(0x8000_073C));

impl DerivationPath<ChimericBip44<Root>> {
    /// use the same "model" of 5 derivation level but instead of starting with the
    /// bip44 Hard Derivation uses the `'1852` (`'0x073C`) derivation path.
    ///
    /// see https://input-output-hk.github.io/adrestia/docs/key-concepts/hierarchical-deterministic-wallets/
    ///
    pub fn chimeric(&self) -> DerivationPath<Bip44<Purpose>> {
        let mut p = self.clone();
        p.push(PURPOSE.into());
        p.coerce_unchecked()
    }
}

impl DerivationPath<ChimericBip44<Purpose>> {
    /// add the next derivation level for the Bip44 chain path derivation.
    ///
    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn coin_type(&self, coin_type: HardDerivation) -> DerivationPath<ChimericBip44<CoinType>> {
        let mut ct = self.clone();
        ct.push(coin_type.into());
        ct.coerce_unchecked()
    }

    #[inline]
    pub fn purpose(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_PURPOSE))
    }
}

impl DerivationPath<ChimericBip44<CoinType>> {
    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn account(&self, account: HardDerivation) -> DerivationPath<ChimericBip44<Account>> {
        let mut a = self.clone();
        a.push(account.into());
        a.coerce_unchecked()
    }

    #[inline]
    pub fn purpose(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_PURPOSE))
    }

    #[inline]
    pub fn coin_type(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_COIN_TYPE))
    }
}

impl DerivationPath<ChimericBip44<Account>> {
    pub const EXTERNAL: SoftDerivation = SoftDerivation::new_unchecked(Derivation::new(0));
    pub const INTERNAL: SoftDerivation = SoftDerivation::new_unchecked(Derivation::new(1));
    pub const ACCOUNT: SoftDerivation = SoftDerivation::new_unchecked(Derivation::new(2));

    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    fn change(&self, change: SoftDerivation) -> DerivationPath<ChimericBip44<Type>> {
        let mut c = self.clone();
        c.push(change.into());
        c.coerce_unchecked()
    }

    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn external(&self) -> DerivationPath<ChimericBip44<Type>> {
        self.change(Self::EXTERNAL)
    }

    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn internal(&self) -> DerivationPath<ChimericBip44<Type>> {
        self.change(Self::INTERNAL)
    }

    /// See [module documentation] for more details
    ///
    /// [module documentation]: ./index.html
    pub fn reward_account(&self) -> DerivationPath<ChimericBip44<Type>> {
        self.change(Self::ACCOUNT)
    }

    #[inline]
    pub fn purpose(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_PURPOSE))
    }

    #[inline]
    pub fn coin_type(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_COIN_TYPE))
    }

    #[inline]
    pub fn account(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_ACCOUNT))
    }
}

impl DerivationPath<ChimericBip44<Type>> {
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
    /// let account = bip44::new().purpose().coin_type(BITCOIN).account(ACCOUNT_01);
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
    ) -> DerivationPathRange<
        DerivationPath<ChimericBip44<Address>>,
        SoftDerivationRange,
        SoftDerivation,
    >
    where
        R: std::ops::RangeBounds<T>,
        T: std::convert::TryInto<SoftDerivation> + Copy,
        <T as std::convert::TryInto<SoftDerivation>>::Error: std::error::Error,
    {
        let range = SoftDerivationRange::new(range);

        self.clone().coerce_unchecked().sub_range(range)
    }

    #[inline]
    pub fn purpose(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_PURPOSE))
    }

    #[inline]
    pub fn coin_type(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_COIN_TYPE))
    }

    #[inline]
    pub fn account(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_ACCOUNT))
    }

    #[inline]
    pub fn change(&self) -> SoftDerivation {
        SoftDerivation::new_unchecked(self.get_unchecked(INDEX_TYPE))
    }
}

impl DerivationPath<ChimericBip44<Address>> {
    #[inline]
    pub fn purpose(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_PURPOSE))
    }

    #[inline]
    pub fn coin_type(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_COIN_TYPE))
    }

    #[inline]
    pub fn account(&self) -> HardDerivation {
        HardDerivation::new_unchecked(self.get_unchecked(INDEX_ACCOUNT))
    }

    #[inline]
    pub fn change(&self) -> SoftDerivation {
        SoftDerivation::new_unchecked(self.get_unchecked(INDEX_TYPE))
    }

    #[inline]
    pub fn address(&self) -> SoftDerivation {
        SoftDerivation::new_unchecked(self.get_unchecked(INDEX_ADDRESS))
    }
}

/* FromStr ***************************************************************** */

macro_rules! mk_from_str_dp_bip44 {
    ($t:ty, $len:expr) => {
        impl FromStr for DerivationPath<$t> {
            type Err = ParseDerivationPathError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let dp = s.parse::<DerivationPath<AnyScheme>>()?;

                if dp.len() == $len {
                    Ok(dp.coerce_unchecked())
                } else {
                    Err(ParseDerivationPathError::InvalidNumberOfDerivations {
                        actual: dp.len(),
                        expected: $len,
                    })
                }
            }
        }
    };
}

mk_from_str_dp_bip44!(Bip44<Purpose>, 1);
mk_from_str_dp_bip44!(Bip44<CoinType>, 2);
mk_from_str_dp_bip44!(Bip44<Account>, 3);
mk_from_str_dp_bip44!(Bip44<Type>, 4);
mk_from_str_dp_bip44!(Bip44<Address>, 5);

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    macro_rules! mk_arbitrary_dp_bip44 {
        ($t:ty, $len:expr) => {
            impl Arbitrary for DerivationPath<$t> {
                fn arbitrary<G: Gen>(g: &mut G) -> Self {
                    let dp = std::iter::repeat_with(|| Derivation::arbitrary(g))
                        .take($len)
                        .collect::<DerivationPath<AnyScheme>>();
                    dp.coerce_unchecked()
                }
            }
        };
    }

    mk_arbitrary_dp_bip44!(Bip44<Purpose>, 1);
    mk_arbitrary_dp_bip44!(Bip44<CoinType>, 2);
    mk_arbitrary_dp_bip44!(Bip44<Account>, 3);
    mk_arbitrary_dp_bip44!(Bip44<Type>, 4);
    mk_arbitrary_dp_bip44!(Bip44<Address>, 5);

    macro_rules! mk_quickcheck_dp_bip44 {
        ($t:ty) => {
            paste::item! {
                #[quickcheck]
                #[allow(non_snake_case)]
                fn [< fmt_parse $t>](derivation_path: DerivationPath<Bip44<$t>>) -> bool {
                    let s = derivation_path.to_string();
                    let v = s.parse::<DerivationPath<Bip44<$t>>>().unwrap();

                    v == derivation_path
                }
            }
        };
    }

    mk_quickcheck_dp_bip44!(Purpose);
    mk_quickcheck_dp_bip44!(CoinType);
    mk_quickcheck_dp_bip44!(Account);
    mk_quickcheck_dp_bip44!(Type);
    mk_quickcheck_dp_bip44!(Address);
}
