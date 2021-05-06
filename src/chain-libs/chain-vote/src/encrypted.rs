use crate::encryption::{Ciphertext, PublicKey};
use crate::gang::Scalar;
use crate::unit_vector::UnitVector;
use rand_core::{CryptoRng, RngCore};

// Power of Two Padded vector structure
#[derive(Clone)]
pub struct PTP<A> {
    pub elements: Vec<A>,
    pub orig_len: usize,
}

impl<A: Clone> PTP<A> {
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn bits(&self) -> usize {
        let len = self.elements.len();
        assert!(len.is_power_of_two());
        len.trailing_zeros() as usize
    }

    pub fn new<F>(mut vec: Vec<A>, extended_value: F) -> PTP<A>
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
        PTP {
            orig_len,
            elements: vec,
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, A> {
        self.elements.iter()
    }
}

impl<A> AsRef<[A]> for PTP<A> {
    fn as_ref(&self) -> &[A] {
        &self.elements
    }
}

#[derive(Clone)]
pub struct EncryptedVote(Vec<Ciphertext>);

/// Encrypted vote is a unit vector where each element is encrypted with ElGamal Ciphertext to
/// the tally opener.
#[derive(Clone)]
pub struct EncryptingVote {
    pub(crate) unit_vector: UnitVector,
    pub ciphertexts: Vec<Ciphertext>,
    pub random_elements: Vec<Scalar>,
}

impl EncryptingVote {
    pub fn prepare<R: RngCore + CryptoRng>(
        rng: &mut R,
        public_key: &PublicKey,
        vote: &UnitVector,
    ) -> Self {
        let mut rs = Vec::new();
        let mut ciphers = Vec::new();
        for vote_element in vote.iter() {
            let (cipher, r) = public_key.encrypt_return_r(&vote_element.into(), rng);
            rs.push(r);
            ciphers.push(cipher);
        }
        Self {
            unit_vector: *vote,
            ciphertexts: ciphers,
            random_elements: rs,
        }
    }

    /*
    pub fn pad<F>(mut self, extended_value: F) -> PTPEncryptingVote
    where
        F: Fn() -> (Scalar, Ciphertext),
    {
        let orig_len = self.ciphertexts.len();

        let expected_len = orig_len.next_power_of_two();
        if orig_len < expected_len {
            let (field_element, zero_cipher) = extended_value();
            while self.ciphertexts.len() < expected_len {
                self.ciphertexts.push(zero_cipher.clone());
                self.random_elements.push(field_element);
            }
        }
        PTPEncryptingVote {
            actual_length: orig_len,
            encrypting_vote: self,
        }
    }
    */
}
