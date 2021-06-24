//! Structures used by the prover of `unit_vector_zkp` during the proof generation. We use the
//! same notation defined in Figure 8

use crate::cryptography::CommitmentKey;
use crate::encrypted_vote::binrep;
use crate::gang::{GroupElement, Scalar};
use crate::math::Polynomial;
use rand_core::{CryptoRng, RngCore};

/// Randomness generated in the proof, used for the hiding property.
pub struct BlindingRandomness {
    alpha: Scalar,
    beta: Scalar,
    gamma: Scalar,
    delta: Scalar,
}

impl BlindingRandomness {
    /// Given a commitment key `ck` and an `index`, compute random `beta`, and return the announcement
    /// corresponding to the commitment of the index, and of `beta`.
    pub fn gen_and_commit<R: RngCore + CryptoRng>(
        ck: &CommitmentKey,
        index: bool,
        rng: &mut R,
    ) -> (Self, Announcement) {
        let (i, alpha) = ck.commit_bool(index, rng);
        let beta = Scalar::random(rng);
        let (b, gamma) = ck.commit(&beta, rng);
        let (a, delta) = if index {
            ck.commit(&beta, rng)
        } else {
            ck.commit(&Scalar::zero(), rng)
        };
        (
            BlindingRandomness {
                alpha,
                beta,
                gamma,
                delta,
            },
            Announcement { i, b, a },
        )
    }

    /// Generate a `ResponseRandomness` from the `BlindingRandomness`, given a `challenge` and `index`.
    pub(crate) fn gen_response(&self, challenge: &Scalar, index: &bool) -> ResponseRandomness {
        let z = Scalar::from(*index) * challenge + &self.beta;
        let w = &self.alpha * challenge + &self.gamma;
        let v = &self.alpha * (challenge - &z) + &self.delta;
        ResponseRandomness { z, w, v }
    }
}

/// First announcement, formed by I, B, A group elements. These group elements
/// are the commitments of the binary representation of the unit vector index.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Announcement {
    pub(crate) i: GroupElement,
    pub(crate) b: GroupElement,
    pub(crate) a: GroupElement,
}

impl Announcement {
    pub const BYTES_LEN: usize = GroupElement::BYTES_LEN * 3;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::BYTES_LEN {
            return None;
        }
        Some(Self {
            i: GroupElement::from_bytes(&bytes[0..GroupElement::BYTES_LEN])?,
            b: GroupElement::from_bytes(
                &bytes[GroupElement::BYTES_LEN..GroupElement::BYTES_LEN * 2],
            )?,
            a: GroupElement::from_bytes(&bytes[GroupElement::BYTES_LEN * 2..])?,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::BYTES_LEN);
        for component in [&self.i, &self.b, &self.a].iter() {
            buf.extend_from_slice(&component.to_bytes());
        }
        debug_assert_eq!(buf.len(), Self::BYTES_LEN);
        buf
    }
}

/// Response encoding the bits of the private vector, and the randomness of `BlindingRandomness`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ResponseRandomness {
    pub(crate) z: Scalar,
    pub(crate) w: Scalar,
    pub(crate) v: Scalar,
}

impl ResponseRandomness {
    pub const BYTES_LEN: usize = Scalar::BYTES_LEN * 3;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::BYTES_LEN {
            return None;
        }
        Some(Self {
            z: Scalar::from_bytes(&bytes[0..Scalar::BYTES_LEN])?,
            w: Scalar::from_bytes(&bytes[Scalar::BYTES_LEN..Scalar::BYTES_LEN * 2])?,
            v: Scalar::from_bytes(&bytes[Scalar::BYTES_LEN * 2..])?,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::BYTES_LEN);
        for component in [&self.z, &self.w, &self.v].iter() {
            buf.extend_from_slice(&component.to_bytes());
        }
        debug_assert_eq!(buf.len(), Self::BYTES_LEN);
        buf
    }
}

/// Generate the polynomials used in Step 5, of the proof generation in Figure 8.
pub(crate) fn generate_polys(
    ciphers_len: usize,
    idx_binary_rep: &[bool],
    bits: usize,
    blinding_randomness_vec: &[BlindingRandomness],
) -> Vec<Polynomial> {
    // Compute polynomials pj(x)
    let polys = idx_binary_rep
        .iter()
        .zip(blinding_randomness_vec.iter())
        .map(|(ix, abcd)| {
            let z1 = Polynomial::new(bits).set2(abcd.beta.clone(), (*ix).into());
            let z0 = Polynomial::new(bits).set2(abcd.beta.negate(), (!ix).into());
            (z0, z1)
        })
        .collect::<Vec<_>>();

    let mut pjs = Vec::new();
    for i in 0..ciphers_len {
        let j = binrep(i, bits as u32);

        let mut acc = if j[0] {
            polys[0].1.clone()
        } else {
            polys[0].0.clone()
        };
        for k in 1..bits {
            let t = if j[k] {
                polys[k].1.clone()
            } else {
                polys[k].0.clone()
            };
            acc = acc * t;
        }
        pjs.push(acc)
    }

    pjs
}
