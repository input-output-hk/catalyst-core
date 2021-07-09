//! Non-interactive Zero Knowledge proof for correct decryption
//! share generation. We use the notation and scheme presented in
//! Figure 12 of the Treasury voting protocol spec.
//!
//! The proof is the following:
//!
//! `NIZK{(pk, C, D), (sk): D = e1^sk AND pk = g^sk}`
//!
//! where `C = (e1, e2)`.
//! This can be translated to the following proof:
//!
//! `NIZK{(g, pk, e1, D), (sk): D = e1^sk AND pk = g^sk}`
//!
//! which is a proof of discrete log equality. We can therefore proof
//! correct decryption using a proof of discrete log equality.
use super::super::dl_equality::DleqZkp;
use crate::cryptography::{Ciphertext, PublicKey, SecretKey};
use crate::GroupElement;
use rand::{CryptoRng, RngCore};

/// Proof of correct decryption.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Zkp {
    vshare_proof: DleqZkp,
}

impl Zkp {
    pub(crate) const PROOF_SIZE: usize = DleqZkp::BYTES_LEN;
    /// Generate a valid share zero knowledge proof.
    pub fn generate<R>(
        c: &Ciphertext,
        pk: &PublicKey,
        share: &GroupElement,
        sk: &SecretKey,
        rng: &mut R,
    ) -> Self
    where
        R: CryptoRng + RngCore,
    {
        let vshare_proof = DleqZkp::generate(
            &GroupElement::generator(),
            &c.e1,
            &pk.pk,
            &share,
            &sk.sk,
            rng,
        );
        Zkp { vshare_proof }
    }

    /// Verify a valid share zero knowledge proof
    pub fn verify(&self, c: &Ciphertext, share: &GroupElement, pk: &PublicKey) -> bool {
        self.vshare_proof
            .verify(&GroupElement::generator(), &c.e1, &pk.pk, &share)
    }

    pub fn to_bytes(&self) -> [u8; Self::PROOF_SIZE] {
        self.vshare_proof.to_bytes()
    }

    #[allow(dead_code)]
    pub fn write_to_bytes(&self, output: &mut [u8]) {
        assert_eq!(output.len(), Self::PROOF_SIZE);
        self.vshare_proof.write_to_bytes(output);
    }

    pub fn from_bytes(slice: &[u8]) -> Option<Self> {
        if slice.len() != Self::PROOF_SIZE {
            return None;
        }
        let vshare_proof = DleqZkp::from_bytes(slice)?;

        let proof = Zkp { vshare_proof };
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

        let plaintext = GroupElement::from_hash(&[0u8]);
        let ciphertext = keypair.public_key.encrypt_point(&plaintext, &mut r);

        let share = &ciphertext.e1 * &keypair.secret_key.sk;

        let proof = Zkp::generate(
            &ciphertext,
            &keypair.public_key,
            &share,
            &keypair.secret_key,
            &mut r,
        );
        let verified = proof.verify(&ciphertext, &share, &keypair.public_key);
        assert!(verified);
    }

    #[test]
    fn serialisation() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);

        let keypair = Keypair::generate(&mut r);

        let plaintext = GroupElement::from_hash(&[0u8]);
        let ciphertext = keypair.public_key.encrypt_point(&plaintext, &mut r);

        let share = &ciphertext.e1 * &keypair.secret_key.sk;

        let proof = Zkp::generate(
            &ciphertext,
            &keypair.public_key,
            &share,
            &keypair.secret_key,
            &mut r,
        );

        let serialised_proof = proof.to_bytes();
        let deseriliased_proof = Zkp::from_bytes(&serialised_proof);
        assert!(deseriliased_proof.is_some());

        assert!(deseriliased_proof
            .unwrap()
            .verify(&ciphertext, &share, &keypair.public_key));
    }
}
