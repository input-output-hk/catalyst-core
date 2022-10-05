//! Non-interactive Zero Knowledge proof for correct ElGamal
//! decryption. We use the notation and scheme presented in
//! Figure 14 of the Treasury voting protocol spec.
//!
//! The proof is the following:
//!
//! `NIZK{(pk, C, M), (sk): M = Dec_sk(C) AND pk = g^sk}`
//!
//! This can be translated to the following proof:
//!
//! `NIZK{(g, pk, e1, (e2 - M)), (sk): (e2 - M) = e1^sk AND pk = g^sk}`
//!
//! which is a proof of discrete log equality. We can therefore prove
//! correct decryption using a proof of discrete log equality.
use crate::cryptography::zkps::dl_equality::DleqZkp;
use crate::cryptography::{Ciphertext, PublicKey, SecretKey};
use crate::GroupElement;
use rand::{CryptoRng, RngCore};

/// Proof of correct decryption.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Zkp {
    dleq_proof: DleqZkp,
}

impl Zkp {
    pub(crate) const PROOF_SIZE: usize = DleqZkp::BYTES_LEN;
    /// Generate a decryption zero knowledge proof.
    pub fn generate<R>(
        c: &Ciphertext,
        pk: &PublicKey,
        message: &GroupElement,
        sk: &SecretKey,
        rng: &mut R,
    ) -> Self
    where
        R: CryptoRng + RngCore,
    {
        let point_2 = &c.e2 - message;
        let dleq_proof = DleqZkp::generate(
            &GroupElement::generator(),
            &c.e1,
            &pk.pk,
            &point_2,
            &sk.sk,
            rng,
        );
        Zkp { dleq_proof }
    }

    /// Verify a decryption zero knowledge proof
    pub fn verify(&self, c: &Ciphertext, m: &GroupElement, pk: &PublicKey) -> bool {
        let point_2 = &c.e2 - m;
        self.dleq_proof
            .verify(&GroupElement::generator(), &c.e1, &pk.pk, &point_2)
    }

    pub fn to_bytes(&self) -> [u8; Self::PROOF_SIZE] {
        let mut output = [0u8; Self::PROOF_SIZE];
        self.write_to_bytes(&mut output);
        output
    }

    pub fn write_to_bytes(&self, output: &mut [u8]) {
        assert_eq!(output.len(), Self::PROOF_SIZE);
        self.dleq_proof.write_to_bytes(output);
    }

    pub fn from_bytes(slice: &[u8]) -> Option<Self> {
        if slice.len() != Self::PROOF_SIZE {
            return None;
        }
        let dleq_proof = DleqZkp::from_bytes(slice)?;

        let proof = Zkp { dleq_proof };
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

        let decryption = keypair.secret_key.decrypt_point(&ciphertext);

        let proof = Zkp::generate(
            &ciphertext,
            &keypair.public_key,
            &decryption,
            &keypair.secret_key,
            &mut r,
        );
        assert!(proof.verify(&ciphertext, &plaintext, &keypair.public_key))
    }

    #[test]
    pub fn serialisation() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);

        let keypair = Keypair::generate(&mut r);

        let plaintext = GroupElement::from_hash(&[0u8]);
        let ciphertext = keypair.public_key.encrypt_point(&plaintext, &mut r);

        let decryption = keypair.secret_key.decrypt_point(&ciphertext);

        let proof = Zkp::generate(
            &ciphertext,
            &keypair.public_key,
            &decryption,
            &keypair.secret_key,
            &mut r,
        );

        let serialised_proof = proof.to_bytes();
        let deserialised_proof = Zkp::from_bytes(&serialised_proof);
        assert!(deserialised_proof.is_some());

        assert!(deserialised_proof
            .unwrap()
            .verify(&ciphertext, &plaintext, &keypair.public_key))
    }
}
