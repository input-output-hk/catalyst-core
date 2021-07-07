use crate::ec::{GroupElement, Scalar};
use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;

/// Challenge context for Discrete Logarithm Equality proof. The common reference string
/// are two EC bases, and the statement consists of two EC points.
/// The challenge computation takes as input the two announcements
/// computed in the sigma protocol, `a1` and `a2`, and the full
/// statement.
pub struct ChallengeContext(Blake2b);

impl ChallengeContext {
    /// Initialise the challenge context, by including the common reference string and the full statement
    pub(crate) fn new(
        base_1: &GroupElement,
        base_2: &GroupElement,
        point_1: &GroupElement,
        point_2: &GroupElement,
    ) -> Self {
        let mut ctx = Blake2b::new(64);
        ctx.input(&base_1.to_bytes());
        ctx.input(&base_2.to_bytes());
        ctx.input(&point_1.to_bytes());
        ctx.input(&point_2.to_bytes());

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
