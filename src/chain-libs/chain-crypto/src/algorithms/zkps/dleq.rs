//! Non-interactive Zero Knowledge proof of Discrete Logarithm
//! EQuality (DLEQ).
//!
//! The proof is the following:
//!
//! `NIZK{(base_1, base_2, point_1, point_2), (dlog): point_1 = base_1^dlog AND point_2 = base_2^dlog}`
//!
//! which makes the statement, the two bases `base_1` and `base_2`, and the two
//! points `point_1` and `point_2`. The witness, on the other hand
//! is the discrete logarithm, `dlog`.
#![allow(clippy::many_single_char_names)]
use super::challenge_context::ChallengeContext;
use crate::ec::ristretto255::{GroupElement, Scalar};
use rand_core::{CryptoRng, RngCore};

/// Proof of correct decryption.
/// Note: if the goal is to reduce the size of a proof, it is better to store the challenge
/// and the response. If on the other hand we want to allow for batch verification of
/// proofs, we should store the announcements and the response.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Zkp {
    challenge: Scalar,
    response: Scalar,
}

impl Zkp {
    pub const BYTES_LEN: usize = 2 * Scalar::BYTES_LEN;
    /// Generate a DLEQ proof
    pub fn generate<R>(
        base_1: &GroupElement,
        base_2: &GroupElement,
        point_1: &GroupElement,
        point_2: &GroupElement,
        dlog: &Scalar,
        rng: &mut R,
    ) -> Self
    where
        R: CryptoRng + RngCore,
    {
        let w = Scalar::random(rng);
        let announcement_1 = base_1 * &w;
        let announcement_2 = base_2 * &w;
        let mut challenge_context = ChallengeContext::new(base_1, base_2, point_1, point_2);
        let challenge = challenge_context.first_challenge(&announcement_1, &announcement_2);
        let response = dlog * &challenge + &w;

        Zkp {
            challenge,
            response,
        }
    }

    /// Verify a DLEQ proof
    pub fn verify(
        &self,
        base_1: &GroupElement,
        base_2: &GroupElement,
        point_1: &GroupElement,
        point_2: &GroupElement,
    ) -> bool {
        let r1 = base_1 * &self.response;
        let r2 = base_2 * &self.response;
        let announcement_1 = r1 - (point_1 * &self.challenge);
        let announcement_2 = r2 - (point_2 * &self.challenge);

        let mut challenge_context = ChallengeContext::new(base_1, base_2, point_1, point_2);
        let challenge = challenge_context.first_challenge(&announcement_1, &announcement_2);
        // no need for constant time equality because of the hash in challenge()
        challenge == self.challenge
    }

    pub fn to_bytes(&self) -> [u8; Self::BYTES_LEN] {
        let mut output = [0u8; Self::BYTES_LEN];
        self.write_to_bytes(&mut output);
        output
    }

    pub fn write_to_bytes(&self, output: &mut [u8]) {
        assert_eq!(output.len(), Self::BYTES_LEN);
        output[0..Scalar::BYTES_LEN].copy_from_slice(&self.challenge.to_bytes());
        output[Scalar::BYTES_LEN..].copy_from_slice(&self.response.to_bytes());
    }

    pub fn from_bytes(slice: &[u8]) -> Option<Self> {
        if slice.len() != Self::BYTES_LEN {
            return None;
        }
        let challenge = Scalar::from_bytes(&slice[..Scalar::BYTES_LEN])?;
        let response = Scalar::from_bytes(&slice[Scalar::BYTES_LEN..])?;

        let proof = Zkp {
            challenge,
            response,
        };
        Some(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    #[test]
    pub fn it_works() {
        let mut r: OsRng = OsRng;

        let dlog = Scalar::random(&mut r);
        let base_1 = GroupElement::from_hash(&[0u8]);
        let base_2 = GroupElement::from_hash(&[0u8]);
        let point_1 = &base_1 * &dlog;
        let point_2 = &base_2 * &dlog;

        let proof = Zkp::generate(&base_1, &base_2, &point_1, &point_2, &dlog, &mut r);

        assert!(proof.verify(&base_1, &base_2, &point_1, &point_2));
    }

    #[test]
    fn serialisation() {
        let mut r: OsRng = OsRng;

        let dlog = Scalar::random(&mut r);
        let base_1 = GroupElement::from_hash(&[0u8]);
        let base_2 = GroupElement::from_hash(&[0u8]);
        let point_1 = &base_1 * &dlog;
        let point_2 = &base_2 * &dlog;

        let proof = Zkp::generate(&base_1, &base_2, &point_1, &point_2, &dlog, &mut r);

        let serialised_proof = proof.to_bytes();
        let deserialised_proof = Zkp::from_bytes(&serialised_proof);

        assert!(deserialised_proof.is_some());

        assert!(deserialised_proof
            .unwrap()
            .verify(&base_1, &base_2, &point_1, &point_2));
    }
}
