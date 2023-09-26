use curve25519_dalek_ng::{
    constants::{RISTRETTO_BASEPOINT_POINT, RISTRETTO_BASEPOINT_TABLE},
    ristretto::{CompressedRistretto, RistrettoPoint as Point},
    scalar::Scalar as IScalar,
    traits::Identity,
};

use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;

#[cfg(any(test, feature = "property-test-api"))]
use proptest::strategy::{BoxedStrategy, Strategy};
use rand_core::{CryptoRng, RngCore};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Mul, Sub};

use curve25519_dalek_ng::traits::{MultiscalarMul, VartimeMultiscalarMul};
use std::array::TryFromSliceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scalar(IScalar);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupElement(Point);

#[cfg(any(test, feature = "property-test-api"))]
impl proptest::arbitrary::Arbitrary for GroupElement {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): ()) -> Self::Strategy {
        proptest::arbitrary::any::<[u8; 64]>()
            .prop_map(|bytes| GroupElement(Point::from_uniform_bytes(&bytes)))
            .boxed()
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for GroupElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_bytes())
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Scalar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.to_bytes())
    }
}

impl GroupElement {
    /// Size of the byte representation of `GroupElement`. We always encode the compressed value
    pub const BYTES_LEN: usize = 32;

    pub fn generator() -> Self {
        GroupElement(RISTRETTO_BASEPOINT_POINT)
    }

    pub fn zero() -> Self {
        GroupElement(Point::identity())
    }

    pub(crate) fn compress(&self) -> CompressedRistretto {
        self.0.compress()
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTES_LEN] {
        self.compress().to_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(GroupElement(
            CompressedRistretto::from_slice(bytes).decompress()?,
        ))
    }

    /// Point from hash
    pub fn from_hash(buffer: &[u8]) -> Self {
        let mut result = [0u8; 64];
        let mut hash = Blake2b::new(64);
        hash.input(buffer);
        hash.result(&mut result);
        GroupElement(Point::from_uniform_bytes(&result))
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

    /// Constant-time multiscalar multiplication using Straus algorithm
    pub fn multiscalar_multiplication<I, J>(scalars: I, points: J) -> Self
    where
        I: IntoIterator<Item = Scalar>,
        J: IntoIterator<Item = GroupElement>,
    {
        GroupElement(Point::multiscalar_mul(
            scalars.into_iter().map(|s| s.0),
            points.into_iter().map(|p| p.0),
        ))
    }

    /// Variable multiscalar multiplication. This function is vulnerable to side-channel attacks and
    /// should only be used when the scalars are not secret.
    pub fn vartime_multiscalar_multiplication<I, J>(scalars: I, points: J) -> Self
    where
        I: IntoIterator<Item = Scalar>,
        J: IntoIterator<Item = GroupElement>,
    {
        GroupElement(Point::vartime_multiscalar_mul(
            scalars.into_iter().map(|s| s.0),
            points.into_iter().map(|p| p.0),
        ))
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
        Scalar(-self.0)
    }

    /// multiplicative inverse
    pub fn inverse(&self) -> Scalar {
        Scalar(self.0.invert())
    }

    /// Increment a
    pub fn increment(&mut self) {
        self.0 += IScalar::one()
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTES_LEN] {
        self.0.to_bytes()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns none if slice is not canonical
    pub fn from_bytes(slice: &[u8]) -> Option<Self> {
        let scalar: Result<[u8; 32], TryFromSliceError> = slice.try_into();
        match scalar {
            Ok(e) => Some(Scalar(IScalar::from_canonical_bytes(e)?)),
            _ => None,
        }
    }

    pub fn hash_to_scalar(b: &Blake2b) -> Scalar {
        let mut h = [0u8; 64];
        b.clone().result(&mut h);
        Scalar(IScalar::from_bytes_mod_order_wide(&h))
    }

    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        Scalar(IScalar::random(rng))
    }

    pub fn from_u64(v: u64) -> Self {
        Scalar(IScalar::from(v))
    }
    /// Raises `x` to the power `n` using binary exponentiation,
    /// with (1 to 2)*lg(n) scalar multiplications.
    /// Not constant time
    pub fn power(&self, n: usize) -> Self {
        let mut result = IScalar::one();
        let mut power = n;
        let mut aux = self.0; // x, x^2, x^4, x^8, ...
        while power > 0 {
            let bit = power & 1;
            if bit == 1 {
                result *= aux;
            }
            power >>= 1;
            aux = aux * aux;
        }
        Scalar(result)
    }

    pub fn sum<I>(mut i: I) -> Option<Self>
    where
        I: Iterator<Item = Self>,
    {
        let mut sum = i.next()?;
        for v in i {
            sum = sum + v;
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
        Scalar(self.0 + other.0)
    }
}

std_ops_gen!(Scalar, Add, Scalar, Scalar, add);

//////////
// FE - FE
//////////

impl<'a, 'b> Sub<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn sub(self, other: &'b Scalar) -> Scalar {
        Scalar(self.0 - other.0)
    }
}

std_ops_gen!(Scalar, Sub, Scalar, Scalar, sub);

//////////
// FE * FE
//////////

impl<'a, 'b> Mul<&'b Scalar> for &'a Scalar {
    type Output = Scalar;

    fn mul(self, other: &'b Scalar) -> Scalar {
        Scalar(self.0 * other.0)
    }
}

std_ops_gen!(Scalar, Mul, Scalar, Scalar, mul);

//////////
// FE * GE
//////////

impl<'a, 'b> Mul<&'b GroupElement> for &'a Scalar {
    type Output = GroupElement;

    fn mul(self, other: &'b GroupElement) -> GroupElement {
        other * self
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a GroupElement {
    type Output = GroupElement;

    fn mul(self, other: &'b Scalar) -> GroupElement {
        if self.0 == RISTRETTO_BASEPOINT_POINT {
            GroupElement(&RISTRETTO_BASEPOINT_TABLE * &other.0)
        } else {
            GroupElement(other.0 * self.0)
        }
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
        let mut a = self.0;
        let mut q = Point::identity();

        while other != 0 {
            if other & 1 != 0 {
                q += a;
            }
            a += a;
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
        GroupElement(self.0 + other.0)
    }
}

std_ops_gen!(GroupElement, Add, GroupElement, GroupElement, add);

//////////
// GE - GE
//////////

impl<'a, 'b> Sub<&'b GroupElement> for &'a GroupElement {
    type Output = GroupElement;

    fn sub(self, other: &'b GroupElement) -> GroupElement {
        GroupElement(self.0 + (-other.0))
    }
}

std_ops_gen!(GroupElement, Sub, GroupElement, GroupElement, sub);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_hash() {
        let element = GroupElement::from_hash(&[1u8]);

        let element2 = GroupElement::from_bytes(&[
            32, 60, 29, 4, 97, 184, 42, 236, 79, 92, 154, 113, 205, 92, 7, 4, 122, 17, 166, 95,
            127, 151, 46, 225, 202, 83, 42, 58, 50, 163, 1, 82,
        ])
        .expect("Point is on curve");

        assert_eq!(element, element2)
    }
}

gen_group_tests!();
