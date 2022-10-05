use crate::testing::{arbitrary::AverageValue, data::Wallet};
use quickcheck::{Arbitrary, Gen};
use std::iter;

#[derive(Clone, Debug)]
pub struct WalletCollection(pub Vec<Wallet>);

impl Arbitrary for WalletCollection {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        let size_limit = 253;
        let n = usize::arbitrary(gen) % size_limit + 1;
        let addresses = iter::from_fn(|| Some(Wallet::arbitrary(gen))).take(n);
        WalletCollection(addresses.collect())
    }
}

impl Arbitrary for Wallet {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        Wallet::from_value(AverageValue::arbitrary(gen).0)
    }
}
