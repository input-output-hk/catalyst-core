use crate::certificate::PoolId;
use crate::chaintypes::ChainLength;
use crate::date::BlockDate;
use crate::fragment::BlockContentHash;

use crate::key::Hash;

/// PraosNonce gathered per block
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PraosNonce([u8; 32]);

impl PraosNonce {
    pub fn zero() -> Self {
        PraosNonce([0u8; 32])
    }

    pub fn from_output_array(array: [u8; 32]) -> Self {
        PraosNonce(array)
    }

    /// Change the nonce to be the result of the hash of the current nonce
    /// and the new supplied nonce.
    ///
    /// Effectively: Self = H(Self, Supplied-Hash)
    pub fn hash_with(&mut self, other: &Self) {
        let mut buf = [0; 64];
        buf[0..32].copy_from_slice(&self.0);
        buf[32..64].copy_from_slice(&other.0);
        self.0.copy_from_slice(Hash::hash_bytes(&buf).as_ref())
    }
}

impl AsRef<[u8]> for PraosNonce {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

/// Genesis Praos related data extract from the header
#[derive(Debug, Clone)]
pub(crate) struct HeaderGPContentEvalContext {
    pub(crate) nonce: PraosNonce,
    pub(crate) pool_creator: PoolId,
}

/// This is the data extracted from a header related to content evaluation
#[derive(Debug, Clone)]
pub struct HeaderContentEvalContext {
    pub(crate) block_date: BlockDate,
    pub(crate) chain_length: ChainLength,
    pub(crate) content_hash: BlockContentHash,
    pub(crate) gp_content: Option<HeaderGPContentEvalContext>,
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};
    use std::iter;

    impl Arbitrary for PraosNonce {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut nonce = [0; 32];
            let nonce_vec: Vec<u8> = iter::from_fn(|| Some(u8::arbitrary(g))).take(32).collect();
            nonce.copy_from_slice(&nonce_vec);
            PraosNonce(nonce)
        }
    }

    impl Arbitrary for HeaderGPContentEvalContext {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            HeaderGPContentEvalContext {
                nonce: Arbitrary::arbitrary(g),
                pool_creator: Arbitrary::arbitrary(g),
            }
        }
    }

    impl Arbitrary for HeaderContentEvalContext {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            HeaderContentEvalContext {
                block_date: Arbitrary::arbitrary(g),
                chain_length: Arbitrary::arbitrary(g),
                gp_content: Arbitrary::arbitrary(g),
                content_hash: Arbitrary::arbitrary(g),
            }
        }
    }
}
