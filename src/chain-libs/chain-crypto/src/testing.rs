use super::*;
use crate::digest;

use quickcheck::{Arbitrary, Gen};
use rand::rngs::SmallRng;
use rand_core::{CryptoRng, Error as RngError, RngCore, SeedableRng};

/// an Arbitrary friendly cryptographic generator
///
/// Given the same generation, all the cryptographic
/// material that is created through it, is
/// deterministic, and thus can be replay
///
/// For obvious reasons, do *not* use anywhere except for testing
#[derive(Clone, Debug)]
pub struct TestCryptoGen(pub u64);

/// A faster non-cryptographic RNG to be used in tests. NOTE: this RNG does
/// implement `CryptoRng`, but it is not really cryptographic. It MUST NOT be
/// used in the production code.
pub struct TestCryptoRng(SmallRng);

impl RngCore for TestCryptoRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RngError> {
        self.0.try_fill_bytes(dest)
    }
}

impl SeedableRng for TestCryptoRng {
    type Seed = <SmallRng as SeedableRng>::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        Self(SmallRng::from_seed(seed))
    }
}

impl CryptoRng for TestCryptoRng {}

impl Arbitrary for TestCryptoGen {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        TestCryptoGen(Arbitrary::arbitrary(g))
    }
}

impl TestCryptoGen {
    /// get the nth deterministic RNG
    pub fn get_rng(&self, idx: u32) -> TestCryptoRng {
        TestCryptoRng::seed_from_u64((idx as u64) * 4096 + self.0)
    }

    /// Get the nth deterministic secret key
    pub fn secret_key<A: AsymmetricKey>(&self, idx: u32) -> SecretKey<A> {
        SecretKey::generate(self.get_rng(idx))
    }

    /// Get the nth deterministic keypair
    pub fn keypair<A: AsymmetricKey>(&self, idx: u32) -> KeyPair<A> {
        KeyPair::from(self.secret_key(idx))
    }
}

#[allow(dead_code)]
pub fn arbitrary_public_key<A: AsymmetricKey, G: Gen>(g: &mut G) -> PublicKey<A::PubAlg> {
    TestCryptoGen::arbitrary(g)
        .keypair::<A>(0)
        .public_key()
        .clone()
}

pub fn arbitrary_secret_key<A, G>(g: &mut G) -> SecretKey<A>
where
    A: AsymmetricKey,
    G: Gen,
{
    TestCryptoGen::arbitrary(g).secret_key(0)
}

#[allow(dead_code)]
pub fn static_secret_key<A>() -> SecretKey<A>
where
    A: AsymmetricKey,
{
    let rng = TestCryptoRng::seed_from_u64(0xfedc_ba98);
    SecretKey::generate(rng)
}

impl<A> Arbitrary for SecretKey<A>
where
    A: AsymmetricKey + 'static,
    A::Secret: Send,
{
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        arbitrary_secret_key(g)
    }
}
impl<A> Arbitrary for KeyPair<A>
where
    A: AsymmetricKey + 'static,
    A::Secret: Send,
    <A::PubAlg as AsymmetricPublicKey>::Public: Send,
{
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let secret_key = SecretKey::arbitrary(g);
        KeyPair::from(secret_key)
    }
}

impl<T, A> Arbitrary for Signature<T, A>
where
    A: VerificationAlgorithm + 'static,
    A::Signature: Send,
    T: Send + 'static,
{
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let bytes: Vec<_> = std::iter::repeat_with(|| u8::arbitrary(g))
            .take(A::SIGNATURE_SIZE)
            .collect();
        Signature::from_binary(&bytes).unwrap()
    }
}

impl Arbitrary for Blake2b256 {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let bytes: Vec<_> = std::iter::repeat_with(|| u8::arbitrary(g))
            .take(Self::HASH_SIZE)
            .collect();
        Self::try_from_slice(&bytes).unwrap()
    }
}

impl<H: digest::DigestAlg + 'static> Arbitrary for digest::Digest<H> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let bytes: Vec<_> = std::iter::repeat_with(|| u8::arbitrary(g))
            .take(26) // actual number doesn't really matter
            .collect();
        digest::Digest::<H>::digest(&bytes[..])
    }
}

impl<H: digest::DigestAlg + 'static, T: 'static> Arbitrary for digest::DigestOf<H, T> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let bytes: Vec<_> = std::iter::repeat_with(|| u8::arbitrary(g))
            .take(26) // actual number doesn't really matter
            .collect();
        digest::DigestOf::<H, Vec<u8>>::digest(&bytes).coerce()
    }
}
