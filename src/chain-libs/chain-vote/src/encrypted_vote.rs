use crate::cryptography::{Ciphertext, UnitVectorZkp};
use crate::tally::ElectionFingerprint;
use crate::{Crs, ElectionPublicKey};
use chain_crypto::ec::Scalar;
/// A vote is represented by a standard basis unit vector of an N dimensional space
///
/// Effectively each possible vote is represented by an axis, where the actual voted option
/// is represented by the unit vector this axis.
///
/// E.g.: given a 3 possible votes in the 0-indexed set {option 0, option 1, option 2}, then
/// the vote "001" represents a vote for "option 2"
pub type Vote = UnitVector;

/// Encrypted vote is a unit vector where each element is an ElGamal Ciphertext, encrypted with
/// the Election Public Key.
pub type EncryptedVote = Vec<Ciphertext>;

/// A proof of correct vote encryption consists of a unit vector zkp, where the voter proves that
/// the `EncryptedVote` is indeed a unit vector, and contains a vote for a single candidate.
pub type ProofOfCorrectVote = UnitVectorZkp;

/// Submitted ballot, which contains an always verified vote.
/// Used for early verification of a vote without requiring additional
/// checks down the chain.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Ballot {
    vote: EncryptedVote,
    // Used to verify that the ballot is applied to the correct
    // encrypted tally
    fingerprint: ElectionFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Invalid vote proof")]
pub struct BallotVerificationError;

impl Ballot {
    pub fn try_from_vote_and_proof(
        vote: EncryptedVote,
        proof: &ProofOfCorrectVote,
        crs: &Crs,
        pk: &ElectionPublicKey,
    ) -> Result<Self, BallotVerificationError> {
        if !proof.verify(crs, &pk.0, &vote) {
            return Err(BallotVerificationError);
        }

        Ok(Self {
            vote,
            fingerprint: (pk, crs).into(),
        })
    }

    pub fn vote(&self) -> &EncryptedVote {
        &self.vote
    }

    pub(super) fn fingerprint(&self) -> &ElectionFingerprint {
        &self.fingerprint
    }
}

/// To achieve logarithmic communication complexity in the unit_vector ZKP, we represent
/// votes as Power of Two Padded vector structures.
#[derive(Clone)]
pub struct Ptp<A> {
    pub elements: Vec<A>,
    pub orig_len: usize,
}

impl<A: Clone> Ptp<A> {
    /// Returns the size of the extended vector
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns the bit size of the extended vector
    pub fn bits(&self) -> usize {
        let len = self.elements.len();
        assert!(len.is_power_of_two());
        len.trailing_zeros() as usize
    }

    /// Generates a new `Ptp` by extending the received `vec` to the next
    /// power of two, padded with `extended_value`.
    pub fn new<F>(mut vec: Vec<A>, extended_value: F) -> Ptp<A>
    where
        A: Clone,
        F: Fn() -> A,
    {
        let orig_len = vec.len();

        let expected_len = orig_len.next_power_of_two();
        if orig_len < expected_len {
            let a = extended_value();
            while vec.len() < expected_len {
                vec.push(a.clone());
            }
        }
        Ptp {
            orig_len,
            elements: vec,
        }
    }

    /// Iterates over the elements
    pub fn iter(&self) -> std::slice::Iter<'_, A> {
        self.elements.iter()
    }
}

impl<A> AsRef<[A]> for Ptp<A> {
    fn as_ref(&self) -> &[A] {
        &self.elements
    }
}

#[derive(Clone, Copy)]
/// Represents a Unit vector which size is @size and the @ith element (0-indexed) is enabled
pub struct UnitVector {
    ith: usize,
    size: usize,
}

impl std::fmt::Debug for UnitVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_{}({})", self.ith, self.size)
    }
}

impl std::fmt::Display for UnitVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_{}({})", self.ith, self.size)
    }
}

// `is_empty` cannot ever be useful in the case of UnitVector,
// as the size will always be > 0 as enforced in new()
#[allow(clippy::len_without_is_empty)]
impl UnitVector {
    /// Create a new `ith` unit vector, with `size` greater than zero, and greater than `ith`.
    pub fn new(size: usize, ith: usize) -> Self {
        assert!(size > 0);
        assert!(ith < size);
        UnitVector { ith, size }
    }

    pub fn iter(&self) -> UnitVectorIter {
        UnitVectorIter(0, *self)
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn ith(&self) -> usize {
        self.ith
    }

    pub fn is_jth(&self, j: usize) -> bool {
        if j >= self.size {
            panic!(
                "out of bounds: unit vector {} accessing index {}",
                self.size, j
            );
        }
        j == self.ith
    }

    pub fn jth(&self, j: usize) -> Scalar {
        if j >= self.size {
            panic!(
                "out of bounds: unit vector {} accessing index {}",
                self.size, j
            );
        } else if j == self.ith {
            Scalar::one()
        } else {
            Scalar::zero()
        }
    }
}

pub fn binrep(n: usize, digits: u32) -> Vec<bool> {
    assert!(n < 2usize.pow(digits));
    (0..digits)
        .rev()
        .map(|i: u32| (n & (1 << i)) != 0)
        .collect::<Vec<bool>>()
}

#[derive(Clone, Copy)]
pub struct UnitVectorIter(usize, UnitVector);

impl Iterator for UnitVectorIter {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.0;
        if i == self.1.size {
            None
        } else {
            self.0 += 1;
            Some(i == self.1.ith)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_vector() {
        let uv = UnitVector::new(5, 0);
        assert_eq!(
            &uv.iter().collect::<Vec<_>>()[..],
            [true, false, false, false, false]
        );
        assert_eq!(
            &uv.iter().collect::<Vec<_>>()[..],
            &(0usize..5).map(|i| uv.is_jth(i)).collect::<Vec<_>>()[..]
        );

        let uv = UnitVector::new(5, 4);
        assert_eq!(
            &uv.iter().collect::<Vec<_>>()[..],
            [false, false, false, false, true]
        );

        assert_eq!(
            &uv.iter().collect::<Vec<_>>()[..],
            &(0usize..5).map(|i| uv.is_jth(i)).collect::<Vec<_>>()[..]
        );
    }

    #[test]
    fn unit_binrep() {
        assert_eq!(binrep(3, 5), &[false, false, false, true, true])
    }
}
