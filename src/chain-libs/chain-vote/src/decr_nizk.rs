//! Non-interactive Zero Knowledge proof for correct ElGamal
//! decryption. We use the notation and scheme presented in
//! Figure 5 of the Treasury voting protocol spec.
//!
//! The proof is the following:
//!
//! `NIZK{(pk, C, M), (sk): M = Dec_sk(C) AND pk = g^sk}`
//!
//! which makes the statement, the public key, `pk`, the ciphertext
//! `(e1, e2)`, and the message, `m`. The witness, on the other hand
//! is the secret key, `sk`.
#![allow(clippy::many_single_char_names)]
use super::encryption::Ciphertext;
use super::gang::{GroupElement, Scalar};
use crate::encryption::{PublicKey, SecretKey};
use cryptoxide::digest::Digest;
use cryptoxide::sha2::Sha512;
use rand::{CryptoRng, RngCore};

/// Proof of correct decryption.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProofDecrypt {
    a1: GroupElement,
    a2: GroupElement,
    z: Scalar,
}

pub(crate) const PROOF_SIZE: usize = 162; // Scalar is 32 bytes

impl ProofDecrypt {
    /// Generate a decryption zero knowledge proof
    pub fn generate<R>(c: &Ciphertext, pk: &PublicKey, sk: &SecretKey, rng: &mut R) -> Self
    where
        R: CryptoRng + RngCore,
    {
        let w = Scalar::random(rng);
        let a1 = GroupElement::generator() * &w;
        let a2 = &c.e1 * &w;
        let d = &c.e1 * &sk.sk;
        let e = challenge(pk, c, &d, &a1, &a2);
        let z = &sk.sk * &e.0 + &w;

        ProofDecrypt { a1, a2, z }
    }

    /// Verify a decryption zero knowledge proof
    pub fn verify(&self, c: &Ciphertext, m: &GroupElement, pk: &PublicKey) -> bool {
        let d = &c.e2 - m;
        let e = challenge(pk, c, &d, &self.a1, &self.a2);
        let gz = GroupElement::generator() * &self.z;
        let he = &pk.pk * &e.0;
        let he_a1 = he + &self.a1;
        let c1z = &c.e1 * &self.z;
        let de = d * &e.0;
        let de_a2 = de + &self.a2;
        gz == he_a1 && c1z == de_a2
    }

    pub fn to_bytes(&self) -> [u8; PROOF_SIZE] {
        let mut output = [0u8; PROOF_SIZE];
        output[0..65].copy_from_slice(&self.a1.to_bytes());
        output[65..130].copy_from_slice(&self.a2.to_bytes());
        output[130..162].copy_from_slice(&self.z.to_bytes());
        output
    }

    pub fn to_slice_mut(&self, output: &mut [u8]) {
        assert_eq!(output.len(), PROOF_SIZE);
        output[0..65].copy_from_slice(&self.a1.to_bytes());
        output[65..130].copy_from_slice(&self.a2.to_bytes());
        output[130..162].copy_from_slice(&self.z.to_bytes());
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != PROOF_SIZE {
            return None;
        }
        let a1 = GroupElement::from_bytes(&slice[0..65])?;
        let a2 = GroupElement::from_bytes(&slice[65..130])?;
        let z = Scalar::from_bytes(&slice[130..162])?;

        let proof = ProofDecrypt { a1, a2, z };
        Some(proof)
    }
}

/// The challenge computation takes as input the two announcements
/// computed in the sigma protocol, `a1` and `a2`, and the full
/// statement.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Challenge(Scalar);

fn challenge(
    pk: &PublicKey,
    c: &Ciphertext,
    d: &GroupElement,
    a1: &GroupElement,
    a2: &GroupElement,
) -> Challenge {
    let mut out = [0u8; 64];
    let mut ctx = Sha512::new();
    ctx.input(&pk.to_bytes());
    ctx.input(&c.to_bytes());
    ctx.input(&d.to_bytes());
    ctx.input(&a1.to_bytes());
    ctx.input(&a2.to_bytes());
    ctx.result(&mut out);
    Challenge(Scalar::from_bytes(&out[0..32]).unwrap())
}

#[cfg(test)]
mod tests {
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    use super::{GroupElement, ProofDecrypt, Scalar};
    use crate::encryption::{Keypair, PublicKey};
    // use chain_crypto::algorithms::vrf::dleq::*;

    #[test]
    pub fn it_works() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);

        let keypair = Keypair::generate(&mut r);

        let plaintext = GroupElement::from_hash(&[0u8]);
        let ciphertext = keypair.public_key.encrypt_point(&plaintext, &mut r);

        let proof = ProofDecrypt::generate(
            &ciphertext,
            &keypair.public_key,
            &keypair.secret_key,
            &mut r,
        );
        let verified = proof.verify(&ciphertext, &plaintext, &keypair.public_key);
        assert_eq!(verified, true);
    }
}
