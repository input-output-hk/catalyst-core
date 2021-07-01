use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use eccoxide::curve::sec2::p256k1::{FieldElement, Point, PointAffine, Scalar as IScalar};
use eccoxide::curve::{Sign as ISign, Sign::Negative, Sign::Positive};
use rand_core::{CryptoRng, RngCore};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scalar(IScalar);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupElement(Point);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coordinate(FieldElement);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sign(ISign);

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for GroupElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_bytes())
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Scalar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_bytes())
    }
}

impl Coordinate {
    pub const BYTES_LEN: usize = FieldElement::SIZE_BYTES;

    pub fn to_bytes(&self) -> [u8; Self::BYTES_LEN] {
        self.0.to_bytes()
    }

    pub fn from_bytes(input: &[u8]) -> Option<Self> {
        if input.len() < Self::BYTES_LEN {
            None
        } else {
            Some(Coordinate(FieldElement::from_slice(
                &input[..Self::BYTES_LEN],
            )?))
        }
    }
}

impl GroupElement {
    /// Size of the byte representation of `GroupElement`.
    pub const BYTES_LEN: usize = 65;

    /// Serialized GroupElement::zero
    const BYTES_ZERO: [u8; Self::BYTES_LEN] = [0; Self::BYTES_LEN];

    /// Point from hash
    pub fn from_hash(buffer: &[u8]) -> Self {
        let mut result = [0u8; 33];
        let mut hash = Blake2b::new(33);
        let mut i = 0u32;
        loop {
            hash.input(buffer);
            hash.input(&i.to_be_bytes());
            hash.result(&mut result);
            hash.reset();
            // arbitrary encoding of sign
            let sign = if result[32] & 1 == 0 {
                Sign(Positive)
            } else {
                Sign(Negative)
            };
            if let Some(point) = Self::from_x_bytes(&result[0..32], sign) {
                break point;
            }
            i += 1;
        }
    }

    fn from_x_bytes(bytes: &[u8], sign: Sign) -> Option<Self> {
        let x_coord = Coordinate::from_bytes(bytes)?;
        Self::decompress(&x_coord, sign)
    }

    pub fn decompress(coord: &Coordinate, sign: Sign) -> Option<Self> {
        Some(GroupElement(Point::from_affine(&PointAffine::decompress(
            &coord.0, sign.0,
        )?)))
    }

    pub fn generator() -> Self {
        GroupElement(Point::generator())
    }

    pub fn zero() -> Self {
        GroupElement(Point::infinity())
    }

    pub fn normalize(&mut self) {
        self.0.normalize()
    }

    pub(crate) fn compress(&self) -> Option<(Coordinate, Sign)> {
        self.0.to_affine().map(|p| {
            let (x, sign) = p.compress();
            (Coordinate(x.clone()), Sign(sign))
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTES_LEN] {
        match self.0.to_affine() {
            None => Self::BYTES_ZERO,
            Some(pa) => {
                let mut bytes = [0u8; Self::BYTES_LEN];
                let (x, y) = pa.to_coordinate();
                bytes[0] = 0x4;
                x.to_slice(&mut bytes[1..33]);
                y.to_slice(&mut bytes[33..65]);
                bytes
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes[0] == 0x4 {
            let x = FieldElement::from_slice(&bytes[1..33])?;
            let y = FieldElement::from_slice(&bytes[33..65])?;
            let p = PointAffine::from_coordinate(&x, &y)?;
            Some(GroupElement(Point::from_affine(&p)))
        } else if bytes == Self::BYTES_ZERO {
            Some(Self::zero())
        } else {
            None
        }
    }

    pub fn sum<'a, I>(i: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        let mut sum = GroupElement::zero();
        for v in i {
            sum = sum + v;
        }
        sum
    }
}

impl Scalar {
    pub const BYTES_LEN: usize = 32;

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

    pub fn to_bytes(&self) -> [u8; Self::BYTES_LEN] {
        self.0.to_bytes()
    }

    pub fn from_bytes(slice: &[u8]) -> Option<Self> {
        IScalar::from_slice(slice).map(Scalar)
    }

    pub fn hash_to_scalar(b: &Blake2b) -> Scalar {
        let mut h = [0u8; 64];
        let mut i = 0u8;
        let mut hash = b.clone();
        loop {
            hash.input(&i.to_be_bytes());
            hash.result(&mut h);
            hash.reset();

            if let Some(scalar) = Self::from_bytes(&h[..32]) {
                break scalar;
            }
            i += 1;
        }
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
        for v in i {
            sum = &sum + &v;
        }
        Some(sum)
    }

    /// Return an iterator of the powers of `x`.
    pub fn exp_iter(&self) -> ScalarExp {
        let next_exp_x = Scalar::one();
        ScalarExp {
            x: self.clone(),
            next_exp_x,
        }
    }
}

/// Provides an iterator over the powers of a `Scalar`.
///
/// This struct is created by the `exp_iter` function.
#[derive(Clone)]
pub struct ScalarExp {
    x: Scalar,
    next_exp_x: Scalar,
}

impl Iterator for ScalarExp {
    type Item = Scalar;

    fn next(&mut self) -> Option<Scalar> {
        let exp_x = self.next_exp_x.clone();
        self.next_exp_x = &self.next_exp_x * &self.x;
        Some(exp_x)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
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

//////////
// FE + FE
//////////

impl<'a, 'b> Add<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn add(self, other: &'b Scalar) -> Scalar {
        Scalar(&self.0 + &other.0)
    }
}

std_ops_gen!(Scalar, Add, Scalar, Scalar, add);

//////////
// FE - FE
//////////

impl<'a, 'b> Sub<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn sub(self, other: &'b Scalar) -> Scalar {
        Scalar(&self.0 - &other.0)
    }
}

std_ops_gen!(Scalar, Sub, Scalar, Scalar, sub);
//////////
// FE * FE
//////////

impl<'a, 'b> Mul<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn mul(self, other: &'b Scalar) -> Scalar {
        Scalar(&self.0 * &other.0)
    }
}

std_ops_gen!(Scalar, Mul, Scalar, Scalar, mul);

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

std_ops_gen!(Scalar, Mul, GroupElement, GroupElement, mul);

std_ops_gen!(GroupElement, Mul, Scalar, GroupElement, mul);

//////////
// u64 * GE
//////////

impl<'a> Mul<&'a GroupElement> for u64 {
    type Output = GroupElement;

    fn mul(self, other: &'a GroupElement) -> GroupElement {
        other * self
    }
}

impl<'a> Mul<u64> for &'a GroupElement {
    type Output = GroupElement;

    fn mul(self, mut other: u64) -> GroupElement {
        let mut a = self.0.clone();
        let mut q = Point::infinity();

        while other != 0 {
            if other & 1 != 0 {
                q = &q + &a;
            }
            a = &a + &a;
            other >>= 1;
        }
        GroupElement(q)
    }
}

//////////
// GE + GE
//////////

impl<'a, 'b> Add<&'b GroupElement> for &'a GroupElement {
    type Output = GroupElement;

    fn add(self, other: &'b GroupElement) -> GroupElement {
        GroupElement(&self.0 + &other.0)
    }
}

std_ops_gen!(GroupElement, Add, GroupElement, GroupElement, add);

//////////
// GE - GE
//////////

impl<'a, 'b> Sub<&'b GroupElement> for &'a GroupElement {
    type Output = GroupElement;

    fn sub(self, other: &'b GroupElement) -> GroupElement {
        GroupElement(&self.0 + (-&other.0))
    }
}

std_ops_gen!(GroupElement, Sub, GroupElement, GroupElement, sub);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_bytes_overflowing() {
        let element = [255u8; 32];

        let try_group_element = GroupElement::from_bytes(&element);

        assert!(try_group_element.is_none())
    }
    #[test]
    fn from_hash() {
        let element = GroupElement::from_hash(&[1u8]);

        let element2 = GroupElement::from_bytes(&[
            4, 13, 166, 126, 45, 249, 4, 248, 227, 194, 159, 100, 48, 62, 165, 72, 101, 155, 168,
            137, 90, 110, 97, 89, 167, 229, 100, 160, 195, 191, 156, 174, 214, 65, 120, 172, 28,
            98, 217, 114, 141, 108, 225, 197, 90, 251, 208, 66, 121, 120, 247, 73, 98, 111, 219,
            172, 181, 134, 49, 239, 108, 91, 149, 243, 218,
        ])
        .expect("This point is on the curve");
        assert_eq!(element, element2);
    }
}
