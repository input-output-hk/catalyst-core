//! Structures used by the prover of `unit_vector_zkp` during the proof generation. We use the
//! same notation defined in Figure 8

use crate::cryptography::CommitmentKey;
use crate::{GroupElement, Scalar};
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
/// Denoting unit-vector's size as N, this method takes 2(N - 2) polynomial multiplications
/// instead of N * (logN - 1) for the direct implementation
pub(crate) fn generate_polys(
    idx_binary_rep: &[bool],
    bits: usize,
    blinding_randomness_vec: &[BlindingRandomness],
) -> Vec<Vec<Scalar>> {
    // Multiplication of an arbitrary-degree polynomial on a degree-1 polynomial with a binary non-const term
    // By being tailored for a given specific type of poly_deg1-multiplier it
    // has better performance than naive polynomials multiplication:
    // at most poly.len() - 1 additions and poly.len() multiplications instead of 2 * poly.len()
    //
    // NOTE: should be replaced with naive polynomial multiplication, if const-time (data-independent)
    // multiplication complexity is needed
    #[inline]
    fn mul(poly: &[Scalar], poly_deg1: &(Scalar, bool)) -> Vec<Scalar> {
        let mut result = poly.iter().map(|p| p * &poly_deg1.0).collect::<Vec<_>>();
        if poly_deg1.1 {
            for i in 0..poly.len() - 1 {
                result[i + 1] = &result[i + 1] + &poly[i];
            }
            result.push(poly.last().unwrap().clone());
        }
        result
    }
    // Binary tree which leaves are the polynomials corresponding to the indices in range [0, bits)
    fn polynomials_bin_tree(
        parent: &[Scalar],
        current_level: usize,
        params: &TreeParams,
    ) -> Vec<Vec<Scalar>> {
        if current_level != params.max_level {
            let next_level = current_level + 1;
            let left_subtree = polynomials_bin_tree(
                &mul(parent, &params.deltas_0[current_level]),
                next_level,
                params,
            );
            let right_subtree = polynomials_bin_tree(
                &mul(parent, &params.deltas_1[current_level]),
                next_level,
                params,
            );
            left_subtree
                .into_iter()
                .chain(right_subtree.into_iter())
                .collect()
        } else {
            vec![parent.to_vec()]
        }
    }
    // Precomputed degree-1 polynomials with values of the Kronecker delta function (for both possible inputs: 0 and 1)
    // and with corresponding beta-randomness for each bit of a given unit vector
    let deltas_0 = (0..bits)
        .map(|i| {
            (
                blinding_randomness_vec[i].beta.clone().negate(),
                !idx_binary_rep[i],
            )
        })
        .collect::<Vec<_>>();

    let deltas_1 = (0..bits)
        .map(|i| (blinding_randomness_vec[i].beta.clone(), idx_binary_rep[i]))
        .collect::<Vec<_>>();

    struct TreeParams {
        max_level: usize,
        deltas_0: Vec<(Scalar, bool)>,
        deltas_1: Vec<(Scalar, bool)>,
    }
    let tp = TreeParams {
        max_level: bits,
        deltas_0,
        deltas_1,
    };
    // Building 2 subtrees from delta_0[0] and delta_1[0] to avoid 2 excessive multiplications with 1 as it would be with
    // polynomials_bin_tree(&[Scalar::one()], 0, &tp)
    let left_subtree = polynomials_bin_tree(
        &[tp.deltas_0[0].0.clone(), Scalar::from(tp.deltas_0[0].1)],
        1,
        &tp,
    );
    let right_subtree = polynomials_bin_tree(
        &[tp.deltas_1[0].0.clone(), Scalar::from(tp.deltas_1[0].1)],
        1,
        &tp,
    );
    left_subtree
        .into_iter()
        .chain(right_subtree.into_iter())
        .collect()
}
