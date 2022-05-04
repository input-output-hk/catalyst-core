use quickcheck::{Arbitrary, Gen};

#[derive(Clone, Debug)]
pub struct Random1to10(pub u64);

impl Arbitrary for Random1to10 {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self(u64::arbitrary(g) % 10 + 1)
    }
}
