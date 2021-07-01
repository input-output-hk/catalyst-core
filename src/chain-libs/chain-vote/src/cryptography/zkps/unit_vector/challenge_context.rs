use super::messages::Announcement;
use crate::cryptography::{Ciphertext, CommitmentKey, PublicKey};
use chain_crypto::ec::Scalar;
use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;

/// Challenge context for the Unit Vector Zero Knowledge Proof. The common reference string
/// is a commitment key, and the statement consists of a public key, and the encryption of each
/// entry of the vector.
pub struct ChallengeContext(Blake2b);

impl ChallengeContext {
    /// Initialise the challenge context, by including the common reference string and the full statement
    pub(crate) fn new(
        commitment_key: &CommitmentKey,
        public_key: &PublicKey,
        ciphers: &[Ciphertext],
    ) -> Self {
        let mut ctx = Blake2b::new(64);
        ctx.input(&commitment_key.to_bytes());
        ctx.input(&public_key.to_bytes());
        for c in ciphers {
            ctx.input(&c.to_bytes());
        }

        ChallengeContext(ctx)
    }

    /// Generation of the `first_challenge`. This challenge is generated after the `Announcement` is "sent". Hence,
    /// we include the latter to the challenge context and generate its corresponding scalar.
    pub(crate) fn first_challenge(&mut self, ibas: &[Announcement]) -> Scalar {
        for iba in ibas {
            self.0.input(&iba.i.to_bytes());
            self.0.input(&iba.b.to_bytes());
            self.0.input(&iba.a.to_bytes());
        }

        Scalar::hash_to_scalar(&self.0)
    }

    /// Generation of the `second_challenge`. This challenge is generated after the encrypted polynomial
    /// coefficients are "sent". Hence, we include the list of ciphertexts to the challenge context and
    /// generate its corresponding scalar.
    pub(crate) fn second_challenge(&mut self, ds: &[Ciphertext]) -> Scalar {
        for d in ds {
            self.0.input(&d.to_bytes())
        }
        Scalar::hash_to_scalar(&self.0)
    }
}
