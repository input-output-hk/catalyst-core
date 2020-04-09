//! # Random Index scheme
//!
//! here the address is supposed to be 2 level only. It is not clear
//! what is supposed to be the structure of the addressing so no assumption
//! is made here.
//!
//! assumptions is that the two level of derivations are: `m / account / address`
//!

use crate::{AnyScheme, Derivation, DerivationPath, ParseDerivationPathError};
use std::str::{self, FromStr};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rindex<P>(std::marker::PhantomData<P>);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Root;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Account;
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address;

const INDEX_ACCOUNT: usize = 0;
const INDEX_ADDRESS: usize = 1;

#[inline]
pub fn new() -> DerivationPath<Rindex<Root>> {
    DerivationPath::new_empty()
}

impl DerivationPath<Rindex<Root>> {
    pub fn account(&self, derivation: Derivation) -> DerivationPath<Rindex<Account>> {
        let mut a = self.clone();
        a.push(derivation);
        a.coerce_unchecked()
    }
}

impl DerivationPath<Rindex<Account>> {
    pub fn address(&self, derivation: Derivation) -> DerivationPath<Rindex<Address>> {
        let mut a = self.clone();
        a.push(derivation);
        a.coerce_unchecked()
    }
}

impl DerivationPath<Rindex<Address>> {
    #[inline]
    pub fn account(&self) -> Derivation {
        self.get_unchecked(INDEX_ACCOUNT)
    }
    #[inline]
    pub fn address(&self) -> Derivation {
        self.get_unchecked(INDEX_ADDRESS)
    }
}

/* FromStr ***************************************************************** */

macro_rules! mk_from_str_dp_rindex {
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

mk_from_str_dp_rindex!(Rindex<Root>, 0);
mk_from_str_dp_rindex!(Rindex<Account>, 1);
mk_from_str_dp_rindex!(Rindex<Address>, 2);

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    macro_rules! mk_arbitrary_dp_rindex {
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

    mk_arbitrary_dp_rindex!(Rindex<Root>, 0);
    mk_arbitrary_dp_rindex!(Rindex<Account>, 1);
    mk_arbitrary_dp_rindex!(Rindex<Address>, 2);

    macro_rules! mk_quickcheck_dp_rindex {
        ($t:ty) => {
            paste::item! {
                #[quickcheck]
                #[allow(non_snake_case)]
                fn [< fmt_parse $t>](derivation_path: DerivationPath<Rindex<$t>>) -> bool {
                    let s = derivation_path.to_string();
                    let v = s.parse::<DerivationPath<Rindex<$t>>>().unwrap();

                    v == derivation_path
                }
            }
        };
    }

    mk_quickcheck_dp_rindex!(Root);
    mk_quickcheck_dp_rindex!(Account);
    mk_quickcheck_dp_rindex!(Address);
}
