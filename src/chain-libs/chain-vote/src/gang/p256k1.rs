use eccoxide::curve::sec2::p256k1::{FieldElement, Point, PointAffine, Scalar as IScalar};
use rand_core::{CryptoRng, RngCore};
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scalar(IScalar);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupElement(Point);

impl GroupElement {
    pub fn generator() -> Self {
        GroupElement(Point::generator())
    }

    pub fn zero() -> Self {
        GroupElement(Point::infinity())
    }

    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let fe = Scalar::random(rng);
        GroupElement(&Point::generator() * &fe.0)
    }

    pub fn random_with_fe<R: RngCore + CryptoRng>(rng: &mut R) -> (Scalar, Self) {
        let fe = Scalar::random(rng);
        let ge = GroupElement(&Point::generator() * &fe.0);
        (fe, ge)
    }

    pub fn normalize(&mut self) {
        self.0.normalize()
    }

    pub fn to_bytes(&self) -> [u8; 65] {
        let mut bytes = [0u8; 65];
        match self.0.to_affine() {
            None => (),
            Some(pa) => {
                let (x, y) = pa.to_coordinate();
                bytes[0] = 0x4;
                x.to_slice(&mut bytes[1..33]);
                y.to_slice(&mut bytes[33..65]);
            }
        };
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes[0] != 0x4 {
            let x = FieldElement::from_slice(&bytes[1..33])?;
            let y = FieldElement::from_slice(&bytes[33..65])?;
            let p = PointAffine::from_coordinate(&x, &y)?;
            Some(GroupElement(Point::from_affine(&p)))
        } else {
            None
        }
    }

    pub fn sum<'a, I>(mut i: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        let mut sum = GroupElement::zero();
        while let Some(v) = i.next() {
            sum = &sum + v;
        }
        sum
    }

    pub fn table(table_size: usize) -> Vec<Self> {
        let mut table = Vec::with_capacity(table_size);

        let gen = GroupElement::generator();
        let mut r = &gen * Scalar::one();

        for _ in 1..table_size + 1 {
            r.normalize();
            let r2 = &r + &gen;
            table.push(r);
            r = r2;
        }
        table
    }
}

impl Scalar {
    /// additive identity
    pub fn zero() -> Self {
        Scalar(IScalar::zero())
    }

    /// multiplicative identity
    pub fn one() -> Self {
        Scalar(IScalar::one())
    }

    pub fn negate(&self) -> Self {
        Scalar(-&self.0)
    }

    /// multiplicative inverse
    pub fn inverse(&self) -> Scalar {
        Scalar(self.0.inverse())
    }

    /// Increment a
    pub fn increment(&mut self) {
        self.0 = &self.0 + IScalar::one()
    }

    /*
    pub fn from_random_bytes(slice: [u8; 64]) -> Self {
        Scalar(IScalar::init_from_wide_bytes(slice))
    }
    */

    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        IScalar::from_bytes(bytes).map(Scalar)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        IScalar::from_slice(slice).map(Scalar)
    }

    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut r = [0u8; 32];
        loop {
            rng.fill_bytes(&mut r[..]);

            if let Some(s) = IScalar::from_bytes(&r) {
                break (Scalar(s));
            }
        }
    }

    pub fn from_u64(v: u64) -> Self {
        Scalar(IScalar::from_u64(v))
    }

    pub fn power(&self, n: usize) -> Self {
        Self(self.0.power_u64(n as u64))
    }

    pub fn sum<I>(mut i: I) -> Option<Self>
    where
        I: Iterator<Item = Self>,
    {
        let mut sum = i.next()?;
        while let Some(v) = i.next() {
            sum = &sum + &v;
        }
        Some(sum)
    }
}

impl From<bool> for Scalar {
    fn from(b: bool) -> Self {
        if b {
            Scalar::one()
        } else {
            Scalar::zero()
        }
    }
}
/*
impl From<usize> for Scalar {
    fn from(sz: usize) -> Self {
        todo!()
    }
}
*/

macro_rules! lref {
    ($lty: ident, $class: ident, $rty: ident, $out: ident, $f: ident) => {
        impl<'a> $class<$rty> for &'a $lty {
            type Output = $out;

            fn $f(self, other: $rty) -> Self::Output {
                self.$f(&other)
            }
        }
    };
}

macro_rules! rref {
    ($lty: ident, $class: ident, $rty: ident, $out: ident, $f: ident) => {
        impl<'b> $class<&'b $rty> for $lty {
            type Output = $out;

            fn $f(self, other: &'b $rty) -> Self::Output {
                (&self).$f(other)
            }
        }
    };
}

macro_rules! nref {
    ($lty: ident, $class: ident, $rty: ident, $out: ident, $f: ident) => {
        impl $class<$rty> for $lty {
            type Output = $out;

            fn $f(self, other: $rty) -> Self::Output {
                (&self).$f(&other)
            }
        }
    };
}

//////////
// FE + FE
//////////

impl<'a, 'b> Add<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn add(self, other: &'b Scalar) -> Scalar {
        Scalar(&self.0 + &other.0)
    }
}

lref!(Scalar, Add, Scalar, Scalar, add);
rref!(Scalar, Add, Scalar, Scalar, add);
nref!(Scalar, Add, Scalar, Scalar, add);

//////////
// FE - FE
//////////

impl<'a, 'b> Sub<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn sub(self, other: &'b Scalar) -> Scalar {
        Scalar(&self.0 - &other.0)
    }
}

lref!(Scalar, Sub, Scalar, Scalar, sub);
rref!(Scalar, Sub, Scalar, Scalar, sub);
nref!(Scalar, Sub, Scalar, Scalar, sub);

//////////
// FE * FE
//////////

impl<'a, 'b> Mul<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn mul(self, other: &'b Scalar) -> Scalar {
        Scalar(&self.0 * &other.0)
    }
}

lref!(Scalar, Mul, Scalar, Scalar, mul);
rref!(Scalar, Mul, Scalar, Scalar, mul);
nref!(Scalar, Mul, Scalar, Scalar, mul);

//////////
// FE * GE
//////////

impl<'a, 'b> Mul<&'b GroupElement> for &'a Scalar {
    type Output = GroupElement;

    fn mul(self, other: &'b GroupElement) -> GroupElement {
        GroupElement(&other.0 * &self.0)
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a GroupElement {
    type Output = GroupElement;

    fn mul(self, other: &'b Scalar) -> GroupElement {
        GroupElement(&other.0 * &self.0)
    }
}

lref!(Scalar, Mul, GroupElement, GroupElement, mul);
rref!(Scalar, Mul, GroupElement, GroupElement, mul);
nref!(Scalar, Mul, GroupElement, GroupElement, mul);

lref!(GroupElement, Mul, Scalar, GroupElement, mul);
rref!(GroupElement, Mul, Scalar, GroupElement, mul);
nref!(GroupElement, Mul, Scalar, GroupElement, mul);

//////////
// GE + GE
//////////

impl<'a, 'b> Add<&'b GroupElement> for &'a GroupElement {
    type Output = GroupElement;

    fn add(self, other: &'b GroupElement) -> GroupElement {
        GroupElement(&self.0 + &other.0)
    }
}

lref!(GroupElement, Add, GroupElement, GroupElement, add);
rref!(GroupElement, Add, GroupElement, GroupElement, add);
nref!(GroupElement, Add, GroupElement, GroupElement, add);

//////////
// GE - GE
//////////

impl<'a, 'b> Sub<&'b GroupElement> for &'a GroupElement {
    type Output = GroupElement;

    fn sub(self, other: &'b GroupElement) -> GroupElement {
        GroupElement(&self.0 + (-&other.0))
    }
}

lref!(GroupElement, Sub, GroupElement, GroupElement, sub);
rref!(GroupElement, Sub, GroupElement, GroupElement, sub);
nref!(GroupElement, Sub, GroupElement, GroupElement, sub);
