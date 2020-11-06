use crate::transaction::Output;
use chain_addr::{Address, Discrimination, Kind};
use quickcheck::{Arbitrary, Gen};
use std::iter;

#[derive(Clone, Debug)]
pub struct OutputsWithoutMultisig(pub Vec<Output<Address>>);

impl Arbitrary for OutputsWithoutMultisig {
    #[allow(clippy::match_like_matches_macro)]
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        let n = usize::arbitrary(gen);
        OutputsWithoutMultisig(
            iter::from_fn(|| Some(Output::arbitrary(gen)))
                .filter(|x| !matches!(x.address.1, Kind::Multisig { .. }))
                .take(n)
                .collect(),
        )
    }
}

impl OutputsWithoutMultisig {
    pub fn set_discrimination(&mut self, discrimination: Discrimination) {
        for output in &mut self.0 {
            output.address.0 = discrimination
        }
    }
}
