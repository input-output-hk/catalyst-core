//! Non-interactive Zero Knowledge proof for correct Hybrid
//! decryption key generation. We use the notation and scheme
//! presented in Figure 5 of the Treasury voting protocol spec.
//!
//! The proof is the following:
//!
//! `NIZK{(pk, C = (C1, C2), D), (sk): D = C1^sk AND pk = g^sk}`
//!
//! which is a proof of discrete log equality. We can therefore prove
//! correct decryption using a proof of discrete log equality.
use crate::cryptography::elgamal::SymmetricKey;
use crate::cryptography::zkps::dl_equality::DleqZkp;
use crate::cryptography::{HybridCiphertext, PublicKey, SecretKey};
use crate::GroupElement;
use rand::{CryptoRng, RngCore};

/// Proof of correct decryption.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Zkp {
    hybrid_dec_key_proof: DleqZkp,
}

impl Zkp {
    pub(crate) const PROOF_SIZE: usize = DleqZkp::BYTES_LEN;
    /// Generate a decryption zero knowledge proof.
    pub fn generate<R>(
        c: &HybridCiphertext,
        pk: &PublicKey,
        symmetric_key: &SymmetricKey,
        sk: &SecretKey,
        rng: &mut R,
    ) -> Self
    where
        R: CryptoRng + RngCore,
    {
        let hybrid_dec_key_proof = DleqZkp::generate(
            &GroupElement::generator(),
            &c.e1,
            &pk.pk,
            &symmetric_key.group_repr,
            &sk.sk,
            rng,
        );
        Zkp {
            hybrid_dec_key_proof,
        }
    }

    /// Verify a decryption zero knowledge proof
    pub fn verify(
        &self,
        c: &HybridCiphertext,
        symmetric_key: &SymmetricKey,
        pk: &PublicKey,
    ) -> bool {
        self.hybrid_dec_key_proof.verify(
            &GroupElement::generator(),
            &c.e1,
            &pk.pk,
            &symmetric_key.group_repr,
        )
    }

    pub fn to_bytes(&self) -> [u8; Self::PROOF_SIZE] {
        let mut output = [0u8; Self::PROOF_SIZE];
        self.write_to_bytes(&mut output);
        output
    }

    pub fn write_to_bytes(&self, output: &mut [u8]) {
        assert_eq!(output.len(), Self::PROOF_SIZE);
        self.hybrid_dec_key_proof.write_to_bytes(output);
    }

    pub fn from_bytes(slice: &[u8]) -> Option<Self> {
        if slice.len() != Self::PROOF_SIZE {
            return None;
        }
        let hybrid_dec_key_proof = DleqZkp::from_bytes(slice)?;

        let proof = Zkp {
            hybrid_dec_key_proof,
        };
        Some(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cryptography::Keypair;
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    #[test]
    pub fn it_works() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);

        let keypair = Keypair::generate(&mut r);

        let plaintext = [10u8; 43];
        let ciphertext = keypair.public_key.hybrid_encrypt(&plaintext, &mut r);

        let decryption_key = keypair.secret_key.recover_symmetric_key(&ciphertext);

        let proof = Zkp::generate(
            &ciphertext,
            &keypair.public_key,
            &decryption_key,
            &keypair.secret_key,
            &mut r,
        );
        assert!(proof.verify(&ciphertext, &decryption_key, &keypair.public_key))
    }

    #[test]
    pub fn serialisation() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);

        let keypair = Keypair::generate(&mut r);

        let plaintext = [10u8; 43];
        let ciphertext = keypair.public_key.hybrid_encrypt(&plaintext, &mut r);

        let decryption_key = keypair.secret_key.recover_symmetric_key(&ciphertext);

        let proof = Zkp::generate(
            &ciphertext,
            &keypair.public_key,
            &decryption_key,
            &keypair.secret_key,
            &mut r,
        );

        let proof_serialised = proof.to_bytes();
        let proof_deserialised = Zkp::from_bytes(&proof_serialised);
        assert!(proof_deserialised.is_some());

        assert!(proof_deserialised.unwrap().verify(
            &ciphertext,
            &decryption_key,
            &keypair.public_key
        ))
    }
}
