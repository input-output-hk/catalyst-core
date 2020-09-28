use crate::gang::{GroupElement, Scalar};
use rand_core::{CryptoRng, RngCore};
use std::ops::{Add, Mul};

/// Pedersen commitment
#[derive(Clone, PartialEq, Eq)]
pub struct Commitment {
    c: GroupElement,
}

#[derive(Clone)]
pub struct CommitmentKey {
    pub h: GroupElement,
}

impl CommitmentKey {
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let h = GroupElement::random(rng);
        CommitmentKey { h }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Validity {
    Valid,
    Invalid,
}

#[derive(Clone)]
pub struct Open {
    m: Scalar,
    r: Scalar,
}

impl Commitment {
    pub fn new_open(ck: &CommitmentKey, o: &Open) -> Self {
        let c = GroupElement::generator() * &o.m + &ck.h * &o.r;
        Commitment { c }
    }

    pub fn new(ck: &CommitmentKey, m: &Scalar, r: &Scalar) -> Self {
        let c = GroupElement::generator() * m + &ck.h * r;
        Commitment { c }
    }

    pub fn verify(&self, ck: &CommitmentKey, o: &Open) -> Validity {
        let other = GroupElement::generator() * &o.m + &ck.h * &o.r;
        if self.c == other {
            Validity::Valid
        } else {
            Validity::Invalid
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.c.to_bytes().to_vec()
    }
}

impl<'a, 'b> Add<&'b Commitment> for &'a Commitment {
    type Output = Commitment;
    fn add(self, rhs: &'b Commitment) -> Self::Output {
        let c = &self.c + &rhs.c;
        Commitment { c }
    }
}

impl<'b> Add<&'b Commitment> for Commitment {
    type Output = Commitment;
    fn add(self, rhs: &'b Commitment) -> Self::Output {
        let c = &self.c + &rhs.c;
        Commitment { c }
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a Commitment {
    type Output = Commitment;
    fn mul(self, rhs: &'b Scalar) -> Self::Output {
        Commitment { c: &self.c * rhs }
    }
}

impl<'a> Mul<Scalar> for &'a Commitment {
    type Output = Commitment;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Commitment { c: &self.c * rhs }
    }
}

impl Mul<Scalar> for Commitment {
    type Output = Commitment;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Commitment { c: &self.c * &rhs }
    }
}
