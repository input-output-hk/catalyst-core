use rand_core::{CryptoRng, RngCore};
use std::ops::{Add, Mul, Sub};
use zerocaf::{edwards::EdwardsPoint, scalar::Scalar as IScalar, traits::Identity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldElement(IScalar);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupElement(EdwardsPoint);

impl GroupElement {
    pub fn generator() -> Self {
        let g: EdwardsPoint = EdwardsPoint {
            X: zerocaf::field::FieldElement([23, 0, 0, 0, 0]),
            Y: zerocaf::field::FieldElement([
                1664892896009688,
                132583819244870,
                812547420185263,
                637811013879057,
                13284180325998,
            ]),
            Z: zerocaf::field::FieldElement([1, 0, 0, 0, 0]),
            T: zerocaf::field::FieldElement([
                4351986304670635,
                4020128726404030,
                674192131526433,
                1158854437106827,
                6468984742885,
            ]),
        };
        //let g = EdwardsPoint::new_from_y_coord(&zerocaf::constants::EDWARDS_D, 0u8.into()).unwrap();
        GroupElement(g)
    }

    pub fn zero() -> Self {
        GroupElement(EdwardsPoint::identity())
    }

    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let fe = FieldElement::random(rng);
        let ge = GroupElement::generator() * &fe;
        ge
    }

    pub fn random_with_fe<R: RngCore + CryptoRng>(rng: &mut R) -> (FieldElement, Self) {
        let fe = FieldElement::random(rng);
        let ge = GroupElement::generator() * &fe;
        (fe, ge)
    }

    pub fn normalize(&mut self) {}

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.compress().to_bytes()
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
        let mut r = &gen * FieldElement::one();

        for _ in 1..table_size + 1 {
            r.normalize();
            let r2 = &r + &gen;
            table.push(r);
            r = r2;
        }
        table
    }
}

impl FieldElement {
    /// additive identity
    pub fn zero() -> Self {
        FieldElement(Scalar::zero())
    }

    /// multiplicative identity
    pub fn one() -> Self {
        FieldElement(Scalar::one())
    }

    pub fn negate(&self) -> Self {
        FieldElement(-&self.0)
    }

    /// multiplicative inverse
    pub fn inverse(&self) -> FieldElement {
        use zerocaf::traits::ops::Pow;
        //let pm2 = Scalar::minus_one() - (Scalar::one() + Scalar::one());
        let pm2 = Scalar::from_bytes(&[
            //0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            //0xff, 0xff, 0xf4, 0x9b, 0x2b, 0xf0, 0xe4, 0x9f, 0x58, 0xd7, 0x26, 0xa9, 0xd3, 0xde,
            //0x35, 0xb7, 0xa1, 0xe5,
            229, 161, 183, 53, 222, 211, 169, 38, 215, 88, 159, 228, 240, 43, 155, 244, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 1,
        ]);
        FieldElement(self.0.pow(&pm2))
        //FieldElement(self.0.inverse())
    }

    /// Increment a
    pub fn increment(&mut self) {
        self.0 = &self.0 + &Scalar::one()
    }

    pub fn from_random_bytes(bytes: [u8; 64]) -> Self {
        // TODO zerocaf from_bytes_wide is unimplemented
        //FieldElement(Scalar::from_bytes_wide(&bytes));

        let mut b = [0u8; 32];
        b.copy_from_slice(&bytes[0..32]);
        b[31] &= 0;
        // 0x3f;
        FieldElement(Scalar::from_bytes(&b))
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        Some(FieldElement(Scalar::from_bytes(bytes)))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 32 {
            return None;
        }
        let mut b = [0u8; 32];
        b.copy_from_slice(slice);
        Self::from_bytes(&b)
    }

    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut r = [0u8; 64];
        rng.fill_bytes(&mut r[..]);
        Self::from_random_bytes(r)
    }

    pub fn from_u64(v: u64) -> Self {
        FieldElement(Scalar::from(&v))
    }

    pub fn power(&self, n: usize) -> Self {
        use zerocaf::traits::ops::Pow;
        let z = Scalar::from(&(n as u64));
        Self(self.0.pow(&z))
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

impl From<bool> for FieldElement {
    fn from(b: bool) -> Self {
        if b {
            FieldElement::one()
        } else {
            FieldElement::zero()
        }
    }
}

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

impl<'a, 'b> Add<&'b FieldElement> for &'a FieldElement {
    type Output = FieldElement;

    fn add(self, other: &'b FieldElement) -> FieldElement {
        FieldElement(&self.0 + &other.0)
    }
}

lref!(FieldElement, Add, FieldElement, FieldElement, add);
rref!(FieldElement, Add, FieldElement, FieldElement, add);
nref!(FieldElement, Add, FieldElement, FieldElement, add);

//////////
// FE - FE
//////////

impl<'a, 'b> Sub<&'b FieldElement> for &'a FieldElement {
    type Output = FieldElement;

    fn sub(self, other: &'b FieldElement) -> FieldElement {
        FieldElement(&self.0 - &other.0)
    }
}

lref!(FieldElement, Sub, FieldElement, FieldElement, sub);
rref!(FieldElement, Sub, FieldElement, FieldElement, sub);
nref!(FieldElement, Sub, FieldElement, FieldElement, sub);

//////////
// FE * FE
//////////

impl<'a, 'b> Mul<&'b FieldElement> for &'a FieldElement {
    type Output = FieldElement;

    fn mul(self, other: &'b FieldElement) -> FieldElement {
        FieldElement(&self.0 * &other.0)
    }
}

lref!(FieldElement, Mul, FieldElement, FieldElement, mul);
rref!(FieldElement, Mul, FieldElement, FieldElement, mul);
nref!(FieldElement, Mul, FieldElement, FieldElement, mul);

//////////
// FE * GE
//////////

impl<'a, 'b> Mul<&'b GroupElement> for &'a FieldElement {
    type Output = GroupElement;

    fn mul(self, other: &'b GroupElement) -> GroupElement {
        GroupElement(&other.0 * &self.0)
    }
}

impl<'a, 'b> Mul<&'b FieldElement> for &'a GroupElement {
    type Output = GroupElement;

    fn mul(self, other: &'b FieldElement) -> GroupElement {
        GroupElement(&self.0 * &other.0)
    }
}

lref!(FieldElement, Mul, GroupElement, GroupElement, mul);
rref!(FieldElement, Mul, GroupElement, GroupElement, mul);
nref!(FieldElement, Mul, GroupElement, GroupElement, mul);

lref!(GroupElement, Mul, FieldElement, GroupElement, mul);
rref!(GroupElement, Mul, FieldElement, GroupElement, mul);
nref!(GroupElement, Mul, FieldElement, GroupElement, mul);

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
        GroupElement(&self.0 + &(-&other.0))
    }
}

lref!(GroupElement, Sub, GroupElement, GroupElement, sub);
rref!(GroupElement, Sub, GroupElement, GroupElement, sub);
nref!(GroupElement, Sub, GroupElement, GroupElement, sub);
