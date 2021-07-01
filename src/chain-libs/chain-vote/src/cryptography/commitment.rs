use crate::tally::Crs;
use chain_crypto::ec::{GroupElement, Scalar};
use rand_core::{CryptoRng, RngCore};

/// Pedersen Commitment key
#[derive(Clone)]
pub struct CommitmentKey {
    pub h: GroupElement,
}

impl CommitmentKey {
    pub fn to_bytes(&self) -> [u8; GroupElement::BYTES_LEN] {
        self.h.to_bytes()
    }

    /// Return a commitment with the given opening, `o`
    pub(crate) fn commit_with_open(&self, o: &Open) -> GroupElement {
        self.commit_with_random(&o.m, &o.r)
    }

    // Return a commitment with the given message, `m`,  and opening key, `r`
    fn commit_with_random(&self, m: &Scalar, r: &Scalar) -> GroupElement {
        GroupElement::generator() * m + &self.h * r
    }

    /// Return a commitment, and the used randomness, `r`, where the latter is computed
    /// from a `Rng + CryptoRng`
    pub(crate) fn commit<R>(&self, m: &Scalar, rng: &mut R) -> (GroupElement, Scalar)
    where
        R: CryptoRng + RngCore,
    {
        let r = Scalar::random(rng);
        (self.commit_with_random(m, &r), r)
    }

    /// Return a commitment of a boolean value, and the used randomness, `r`, where the latter is computed
    /// from a `Rng + CryptoRng`
    pub(crate) fn commit_bool<R>(&self, m: bool, rng: &mut R) -> (GroupElement, Scalar)
    where
        R: CryptoRng + RngCore,
    {
        let r = Scalar::random(rng);
        if m {
            (GroupElement::generator() + &self.h * &r, r)
        } else {
            (&self.h * &r, r)
        }
    }

    /// Verify that a given `commitment` opens to `o` under commitment key `self`
    #[allow(dead_code)]
    pub fn verify(&self, commitment: &GroupElement, o: &Open) -> bool {
        let other = self.commit_with_open(o);
        commitment == &other
    }
}

impl From<Crs> for CommitmentKey {
    fn from(crs: Crs) -> Self {
        CommitmentKey { h: crs }
    }
}

#[derive(Clone)]
pub struct Open {
    pub m: Scalar,
    pub r: Scalar,
}

#[cfg(tests)]
mod tests {
    use super::*;
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn commit_and_open() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
        let crs = Crs::from_hash(&[0u8]);
        let commitment_key = CommitmentKey::from(crs);
        let message = Scalar::random(&mut rng);
        let (comm, rand) = commitment_key.commit(&message, &mut rng);

        let opening = Open {
            m: message,
            r: rand,
        };

        assert!(commitment_key.verify(&comm, &opening));

        let comm_with_rand = commitment_key.commit_with_random(&message, &rand);

        assert_eq!(comm_with_rand, comm);

        let comm_with_open = commitment_key.commit_with_open(&opening);

        assert_eq!(comm_with_open, comm);
    }
}
