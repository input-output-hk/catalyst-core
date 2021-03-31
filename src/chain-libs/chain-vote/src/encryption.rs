#![allow(dead_code)]

//! Implementation of the different encryption/decryption mechanisms used in `chain-vote`, including their
//! corresponding structures. In particular, we use (lifted) ElGamal cryptosystem, and combine with ChaCha
//! stream cipher to produce a hybrid encryption scheme.

use crate::gang::{GroupElement, Scalar};
use rand_core::{CryptoRng, RngCore};
use std::ops::{Add, Mul};

use cryptoxide::blake2b::Blake2b;
use cryptoxide::chacha20::ChaCha20;
use cryptoxide::digest::Digest;

#[derive(Debug, Clone, Eq, PartialEq)]
/// ElGamal public key. pk = sk * G, where sk is the `SecretKey` and G is the group
/// generator.
pub struct PublicKey {
    pub pk: GroupElement,
}

#[derive(Clone)]
/// ElGamal secret key
pub struct SecretKey {
    pub sk: Scalar,
}

#[derive(Clone)]
/// ElGamal keypair
pub struct Keypair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// ElGamal ciphertext. Given a message M represented by a group element, and ElGamal
/// ciphertext consists of (r * G; M + r * `PublicKey`), where r is a random `Scalar`.
pub struct Ciphertext {
    e1: GroupElement,
    e2: GroupElement,
}

#[derive(Clone)]
pub struct HybridCiphertext {
    // ElGamal Ciphertext
    e1: Ciphertext,
    // Symmetric encrypted message
    e2: Box<[u8]>,
}

/// The hybrid encryption scheme uses a group element as a
/// representation of the symmetric key. This facilitates
/// its exchange using ElGamal encryption.
pub struct SymmetricKey {
    group_repr: GroupElement,
}

impl PublicKey {
    pub const BYTES_LEN: usize = GroupElement::BYTES_LEN;

    pub fn to_bytes(&self) -> Vec<u8> {
        self.pk.to_bytes().to_vec()
    }

    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        Some(Self {
            pk: GroupElement::from_bytes(buf)?,
        })
    }

    /// Given a `message` represented as a group element, return a ciphertext.
    fn encrypt_point<R>(&self, message: &GroupElement, rng: &mut R) -> Ciphertext
    where
        R: RngCore + CryptoRng,
    {
        let r = Scalar::random(rng);
        self.encrypt_point_with_r(&message, &r)
    }

    /// Given a `message` represented as a group element, return a ciphertext and the
    /// randomness used.
    fn encrypt_point_return_r<R>(&self, message: &GroupElement, rng: &mut R) -> (Ciphertext, Scalar)
    where
        R: RngCore + CryptoRng,
    {
        let r = Scalar::random(rng);
        (self.encrypt_point_with_r(&message, &r), r)
    }

    /// Given a `message` represented as a group element, and some value used as `randomness`,
    /// return the corresponding ciphertext. This function should only be called when the
    /// randomness value needs to be a particular value (e.g. verification procedure of the unit vector ZKP).
    /// Otherwise, `encrypt_point` should be used.
    fn encrypt_point_with_r(&self, message: &GroupElement, randomness: &Scalar) -> Ciphertext {
        Ciphertext {
            e1: &GroupElement::generator() * randomness,
            e2: message + &(&self.pk * randomness),
        }
    }

    /// Given a `message` represented as a `Scalar`, return a ciphertext using the
    /// "lifted ElGamal" mechanism. Mainly, return (r * G; `message` * G + r * `self`)
    pub(crate) fn encrypt<R>(&self, message: &Scalar, rng: &mut R) -> Ciphertext
    where
        R: RngCore + CryptoRng,
    {
        self.encrypt_point(&(&GroupElement::generator() * message), rng)
    }

    /// Given a `message` represented as a `Scalar`, return a ciphertext and return
    /// the randomness used.
    pub(crate) fn encrypt_return_r<R>(&self, message: &Scalar, rng: &mut R) -> (Ciphertext, Scalar)
    where
        R: RngCore + CryptoRng,
    {
        self.encrypt_point_return_r(&(&GroupElement::generator() * message), rng)
    }

    /// Given a `message` represented as a `Scalar`, and some value used as `randomness`,
    /// return the corresponding ciphertext. This function should only be called when the
    /// randomness value is not random (e.g. verification procedure of the unit vector ZKP).
    /// Otherwise, `encrypt_point` should be used.
    pub(crate) fn encrypt_with_r(&self, message: &Scalar, randomness: &Scalar) -> Ciphertext {
        self.encrypt_point_with_r(&(&GroupElement::generator() * message), randomness)
    }

    /// Given a `message` passed as bytes, encrypt it using hybrid encryption.
    pub(crate) fn hybrid_encrypt<R>(&self, message: &[u8], rng: &mut R) -> HybridCiphertext
    where
        R: RngCore + CryptoRng,
    {
        let symmetric_key = SymmetricKey::new(rng);
        let e1 = self.encrypt_point(&symmetric_key.group_repr, rng);
        let e2 = symmetric_key.process(message).into_boxed_slice();
        HybridCiphertext { e1, e2 }
    }
}

impl SecretKey {
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let sk = Scalar::random(rng);
        Self { sk }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Scalar::from_bytes(bytes).map(|sk| Self { sk })
    }

    #[allow(dead_code)]
    /// Decrypt a message using hybrid decryption
    pub(crate) fn hybrid_decrypt(&self, ciphertext: &HybridCiphertext) -> Vec<u8> {
        let symmetric_key = SymmetricKey {
            group_repr: self.decrypt_point(&ciphertext.e1),
        };

        symmetric_key.process(&ciphertext.e2)
    }

    /// Decrypt ElGamal `Ciphertext` = (`cipher`.e1, `cipher`.e2), by computing
    /// `cipher`.e2 - `self` * `cipher`.e1. This returns the plaintext respresented
    /// as a `GroupElement`.
    pub(crate) fn decrypt_point(&self, cipher: &Ciphertext) -> GroupElement {
        &(&cipher.e1 * &self.sk.negate()) + &cipher.e2
    }
}

impl SymmetricKey {
    /// Generate a new random symmetric key
    pub fn new<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let exponent = Scalar::random(rng);
        SymmetricKey {
            group_repr: GroupElement::generator() * &exponent,
        }
    }

    /// Initialise encryption, by hashing the group element
    fn initialise_encryption(&self) -> ChaCha20 {
        let mut out = [0u8; 44];
        let mut h = Blake2b::new(44);
        h.input(&self.group_repr.to_bytes());
        h.result(&mut out);
        ChaCha20::new(&out[0..32], &out[32..44])
    }

    /// Encrypt/decrypt a message using the symmetric key
    fn process(&self, m: &[u8]) -> Vec<u8> {
        let mut key = self.initialise_encryption();
        let mut dat = m.to_vec();
        key.process_mut(&mut dat);
        dat
    }
}

impl Keypair {
    #[allow(dead_code)]
    pub fn from_secretkey(secret_key: SecretKey) -> Self {
        let public_key = PublicKey {
            pk: &GroupElement::generator() * &secret_key.sk,
        };
        Keypair {
            secret_key,
            public_key,
        }
    }

    /// Generate a keypair for encryption
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Keypair {
        let sk = Scalar::random(rng);
        let pk = &GroupElement::generator() * &sk;
        Keypair {
            secret_key: SecretKey { sk },
            public_key: PublicKey { pk },
        }
    }
}

impl Ciphertext {
    /// Size of the byte representation of `Ciphertext`.
    pub const BYTES_LEN: usize = GroupElement::BYTES_LEN * 2;

    /// the zero ciphertext
    pub fn zero() -> Self {
        Ciphertext {
            e1: GroupElement::zero(),
            e2: GroupElement::zero(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut r = Vec::with_capacity(Self::BYTES_LEN);
        r.extend_from_slice(self.e1.to_bytes().as_ref());
        r.extend_from_slice(self.e2.to_bytes().as_ref());
        debug_assert_eq!(r.len(), Self::BYTES_LEN);
        r
    }

    pub fn from_bytes(slice: &[u8]) -> Option<Ciphertext> {
        let e1 = GroupElement::from_bytes(&slice[..GroupElement::BYTES_LEN])?;
        let e2 = GroupElement::from_bytes(&slice[GroupElement::BYTES_LEN..])?;
        Some(Ciphertext { e1, e2 })
    }

    pub fn elements(&self) -> (&GroupElement, &GroupElement) {
        (&self.e1, &self.e2)
    }
}

impl<'a, 'b> Add<&'b Ciphertext> for &'a Ciphertext {
    type Output = Ciphertext;

    fn add(self, other: &'b Ciphertext) -> Ciphertext {
        Ciphertext {
            e1: &self.e1 + &other.e1,
            e2: &self.e2 + &other.e2,
        }
    }
}

std_ops_gen!(Ciphertext, Add, Ciphertext, Ciphertext, add);

impl<'a, 'b> Mul<&'b Scalar> for &'a Ciphertext {
    type Output = Ciphertext;
    fn mul(self, rhs: &'b Scalar) -> Self::Output {
        Ciphertext {
            e1: &self.e1 * rhs,
            e2: &self.e2 * rhs,
        }
    }
}

std_ops_gen!(Ciphertext, Mul, Scalar, Ciphertext, mul);

impl<'a> Mul<u64> for &'a Ciphertext {
    type Output = Ciphertext;
    fn mul(self, rhs: u64) -> Self::Output {
        Ciphertext {
            e1: &self.e1 * rhs,
            e2: &self.e2 * rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    #[test]
    fn zero() {
        let cipher = Ciphertext {
            e1: GroupElement::zero(),
            e2: GroupElement::zero(),
        };
        assert_eq!(Ciphertext::zero(), cipher)
    }

    #[test]
    fn encrypt_decrypt_point() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        for n in 1..5 {
            let keypair = Keypair::generate(&mut rng);
            let m = GroupElement::generator() * Scalar::from_u64(n * 24);
            let cipher = keypair.public_key.encrypt_point(&m, &mut rng);
            let r = keypair.secret_key.decrypt_point(&cipher);
            assert_eq!(m, r)
        }
    }

    #[test]
    fn encrypt_decrypt() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        for n in 1..5 {
            let keypair = Keypair::generate(&mut rng);
            let m = Scalar::from_u64(n * 24);
            let cipher = keypair.public_key.encrypt(&m, &mut rng);
            let r = keypair.secret_key.decrypt_point(&cipher);
            assert_eq!(m * GroupElement::generator(), r)
        }
    }

    #[test]
    fn symmetric_encrypt_decrypt() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let k = SecretKey::generate(&mut rng);
        let k = Keypair::from_secretkey(k);

        let m = [1, 3, 4, 5, 6, 7];

        let encrypted = &k.public_key.hybrid_encrypt(&m, &mut rng);
        let result = &k.secret_key.hybrid_decrypt(&encrypted);

        assert_eq!(&m[..], &result[..])
    }
}
