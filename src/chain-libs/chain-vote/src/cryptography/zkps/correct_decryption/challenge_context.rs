use crate::cryptography::{Ciphertext, PublicKey};
use chain_crypto::ec::{GroupElement, Scalar};
use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;

/// Challenge context for Decryption Zero Knowledge Proof. The common reference string
/// is a public key, and the statement consists of a ciphertext, and a plaintext.
/// computation takes as input the two announcements
/// computed in the sigma protocol, `a1` and `a2`, and the full
/// statement.
pub struct ChallengeContext(Blake2b);

impl ChallengeContext {
    /// Initialise the challenge context, by including the common reference string and the full statement
    pub(crate) fn new(
        public_key: &PublicKey,
        ciphertext: &Ciphertext,
        plaintext: &GroupElement,
    ) -> Self {
        let mut ctx = Blake2b::new(64);
        ctx.input(&public_key.to_bytes());
        ctx.input(&ciphertext.to_bytes());
        ctx.input(&plaintext.to_bytes());

        ChallengeContext(ctx)
    }

    /// Generation of the `first_challenge`. This challenge is generated after the `Announcement` is
    /// "sent". Hence, we include the latter to the challenge context and generate its
    /// corresponding scalar.
    pub(crate) fn first_challenge(&mut self, a1: &GroupElement, a2: &GroupElement) -> Scalar {
        self.0.input(&a1.to_bytes());
        self.0.input(&a2.to_bytes());

        Scalar::hash_to_scalar(&self.0)
    }
}
