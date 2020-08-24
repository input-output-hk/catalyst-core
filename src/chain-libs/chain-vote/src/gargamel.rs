#![allow(dead_code)]

use crate::gang::{GroupElement, Scalar};
use rand_core::{CryptoRng, RngCore};
use std::ops::{Add, Mul};

// ElGamal Ciphertext
#[derive(Clone)]
pub struct PublicKey {
    pub pk: GroupElement,
}

#[derive(Clone)]
pub struct SecretKey {
    pub sk: Scalar,
}

#[derive(Clone)]
pub struct Keypair {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext {
    e1: GroupElement,
    e2: GroupElement,
}

impl PublicKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.pk.to_bytes().to_vec()
    }
}

impl SecretKey {
    pub fn generate<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let sk = Scalar::random(rng);
        Self { sk }
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
}

impl Ciphertext {
    /// the zero ciphertext
    pub fn zero() -> Self {
        Ciphertext {
            e1: GroupElement::zero(),
            e2: GroupElement::zero(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut r = self.e1.to_bytes().to_vec();
        r.extend_from_slice(&self.e2.to_bytes());
        r
    }

    pub fn from_bytes(slice: &[u8]) -> Option<Ciphertext> {
        let l = slice.len() / 2;
        let e1 = GroupElement::from_bytes(&slice[0..l])?;
        let e2 = GroupElement::from_bytes(&slice[l..])?;
        Some(Ciphertext { e1, e2 })
    }

    pub fn elements(&self) -> (&GroupElement, &GroupElement) {
        (&self.e1, &self.e2)
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

pub fn encrypt_point(pk: &PublicKey, n: &GroupElement, r: &Scalar) -> Ciphertext {
    Ciphertext {
        e1: &GroupElement::generator() * r,
        e2: n + &(&pk.pk * r),
    }
}

pub fn encrypt(pk: &PublicKey, m: &Scalar, r: &Scalar) -> Ciphertext {
    encrypt_point(pk, &(&GroupElement::generator() * m), r)
}

pub fn decrypt_point(sk: &SecretKey, cipher: &Ciphertext) -> GroupElement {
    &(&cipher.e1 * &sk.sk.negate()) + &cipher.e2
}

pub fn decrypt(sk: &SecretKey, cipher: &Ciphertext, incr: usize) -> Option<Scalar> {
    let ge = decrypt_point(sk, cipher);

    let gen = GroupElement::generator();
    let mut r = &gen * Scalar::one();

    for i in 1..incr {
        if r == ge {
            return Some(Scalar::from_u64(i as u64));
        } else {
            r = &r + &gen;
        }
    }
    None
}

pub fn vec_sum(vec_ciphertexts: Vec<Vec<Ciphertext>>) -> Vec<Ciphertext> {
    assert!(vec_ciphertexts.len() > 0);

    let mut result = vec_ciphertexts[0].clone();
    for ciphertexts in &vec_ciphertexts[1..] {
        assert_eq!(ciphertexts.len(), result.len());

        for (r, c) in result.iter_mut().zip(ciphertexts) {
            *r = &*r + c
        }
    }
    result
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

impl<'a> Mul<Scalar> for &'a Ciphertext {
    type Output = Ciphertext;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Ciphertext {
            e1: &self.e1 * &rhs,
            e2: &self.e2 * &rhs,
        }
    }
}

impl<'a, 'b> Mul<&'b Scalar> for &'a Ciphertext {
    type Output = Ciphertext;
    fn mul(self, rhs: &'b Scalar) -> Self::Output {
        Ciphertext {
            e1: &self.e1 * rhs,
            e2: &self.e2 * rhs,
        }
    }
}

impl Mul<Scalar> for Ciphertext {
    type Output = Ciphertext;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Ciphertext {
            e1: &self.e1 * &rhs,
            e2: &self.e2 * &rhs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;
    /*
    use smoke::{
        generator::{self, BoxGenerator},
        Generator,
    };

    fn rng_generator() -> BoxGenerator<ChaCha20Rng> {
        generator::Array32::new(generator::num::<u8>())
            .map(|seed| ChaCha20Rng::from_seed(seed))
            .into_boxed()
    }
    */

    #[test]
    fn zero() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let keypair = generate(&mut rng);
        let m = Scalar::zero();
        let r = Scalar::zero();
        let cipher = encrypt(&keypair.public_key, &m, &r);
        assert_eq!(Ciphertext::zero(), cipher)
    }

    #[test]
    fn encrypt_decrypt_point() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        for n in 1..5 {
            let keypair = generate(&mut rng);
            let m = GroupElement::generator() * Scalar::from_u64(n * 24);
            let r = Scalar::random(&mut rng);
            let cipher = encrypt_point(&keypair.public_key, &m, &r);
            let r = decrypt_point(&keypair.secret_key, &cipher);
            assert_eq!(m, r)
        }
    }

    #[test]
    fn encrypt_decrypt() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        for n in 1..5 {
            let keypair = generate(&mut rng);
            let m = Scalar::from_u64(n * 24);
            let r = Scalar::random(&mut rng);
            let cipher = encrypt(&keypair.public_key, &m, &r);
            let r = decrypt(&keypair.secret_key, &cipher, 5 * 24);
            assert_eq!(Some(m), r)
        }

        /*
        use smoke::{forall, property, run, Testable};
        run(|ctx| {
            forall(rng_generator())
                .ensure(|rng| {
                    let mut rng = rng.clone();
                    let (secret_key, public_key) = generate(&mut rng);
                    let m = Scalar::one();
                    let r = Scalar::random(&mut rng);
                    let cipher = encrypt(&public_key, &m, &r);
                    let r = decrypt(&secret_key, &cipher, 2);
                    property::equal(Some(m), r)
                })
                .test(ctx);
        })
        */
    }
}
