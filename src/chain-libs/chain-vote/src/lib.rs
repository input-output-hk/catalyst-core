//! Chain Vote

mod commitment;
pub mod committee;
mod encrypted;
mod gang;
pub mod gargamel;
mod hybrid;
mod math;
mod shvzk;
mod unit_vector;

// re-export under a debug module
#[doc(hidden)]
pub mod debug {
    pub mod gang {
        pub use crate::gang::*;
    }
    pub mod shvzk {
        pub use crate::shvzk::*;
    }
}

use rand_core::{CryptoRng, RngCore};

pub use committee::{MemberCommunicationKey, MemberCommunicationPublicKey, MemberState};
pub use encrypted::EncryptingVote;
use gang::{Scalar, GROUP_ELEMENT_BYTES_LEN};
pub use gargamel::{Ciphertext, CIPHERTEXT_BYTES_LEN};
pub use unit_vector::UnitVector;

/// Secret key for opening vote
pub type OpeningVoteKey = committee::MemberSecretKey;

/// Public Key for the vote
pub type EncryptingVoteKey = committee::ElectionPublicKey;

/// A vote is represented by a standard basis unit vector of a N dimension space
///
/// Effectively each possible vote is represented by an axis, where the actual voted option
/// is represented by a represented of this axis.
///
/// so given a 3 possible votes in the 0-indexed set {option 0, option 1, option 2}, then
/// the vote "001" represent a vote for "option 2"
pub type Vote = UnitVector;

/// Encrypted vote is a unit vector where each element is encrypted with ElGamal Ciphertext to
/// the tally opener.
pub type EncryptedVote = Vec<Ciphertext>;

pub type ProofOfCorrectVote = shvzk::Proof;

/// Common Reference String
pub type CRS = committee::CRS;

/// Take a vote and encrypt it + provide a proof of correct voting
pub fn encrypt_vote<R: RngCore + CryptoRng>(
    rng: &mut R,
    public_key: &EncryptingVoteKey,
    vote: Vote,
) -> (EncryptedVote, ProofOfCorrectVote) {
    let ev = EncryptingVote::prepare(rng, &public_key.0, &vote);
    let proof = shvzk::prove(rng, &public_key.0, ev.clone());
    (ev.ciphertexts, proof)
}

/// Verify that the encrypted vote is valid without opening it
#[allow(clippy::ptr_arg)]
pub fn verify_vote(
    public_key: &EncryptingVoteKey,
    vote: &EncryptedVote,
    proof: &ProofOfCorrectVote,
) -> bool {
    shvzk::verify(&public_key.0, vote, proof)
}

/// The encrypted tally
#[derive(Clone)]
pub struct Tally {
    r: Vec<Ciphertext>,
}

#[derive(Clone)]
pub struct TallyDecryptShare {
    r1s: Vec<gang::GroupElement>,
}

#[derive(Clone)]
pub struct TallyState {
    pub r2s: Vec<gang::GroupElement>,
}

#[derive(Debug, Clone)]
pub struct TallyResult {
    pub votes: Vec<Option<u64>>,
}

impl Tally {
    /// Start a new tally with N different options
    pub fn new(options: usize) -> Self {
        let r = vec![Ciphertext::zero(); options];
        Tally { r }
    }

    /// Add an encrypted vote with a specific weight to the tally
    ///
    /// Note that the encrypted vote needs to have the exact same number of
    /// options as the tally expect otherwise an assert will trigger
    #[allow(clippy::ptr_arg)]
    pub fn add(&mut self, vote: &EncryptedVote, weight: u64) {
        assert_eq!(vote.len(), self.r.len());
        for (ri, ci) in self.r.iter_mut().zip(vote.iter()) {
            *ri = &*ri + &(ci * Scalar::from_u64(weight));
        }
    }

    pub fn finish(&self, secret_key: &OpeningVoteKey) -> (TallyState, TallyDecryptShare) {
        let mut dshares = Vec::with_capacity(self.r.len());
        let mut r2s = Vec::with_capacity(self.r.len());
        for r in &self.r {
            let (r1, r2) = r.elements();
            dshares.push(r1 * &secret_key.0.sk);
            r2s.push(r2.clone());
        }
        (TallyState { r2s }, TallyDecryptShare { r1s: dshares })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        use std::io::Write;
        let mut bytes: Vec<u8> = Vec::with_capacity(CIPHERTEXT_BYTES_LEN * self.r.len());
        for ri in &self.r {
            bytes.write_all(ri.to_bytes().as_ref()).unwrap();
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() % CIPHERTEXT_BYTES_LEN != 0 {
            return None;
        }
        let r = bytes
            .chunks(CIPHERTEXT_BYTES_LEN)
            .map(Ciphertext::from_bytes)
            .collect::<Option<Vec<_>>>()?;
        Some(Self { r })
    }
}

impl TallyDecryptShare {
    pub fn to_bytes(&self) -> Vec<u8> {
        group_elements_to_bytes(&self.r1s)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        group_elements_from_bytes(bytes).map(|r1s| Self { r1s })
    }
}

impl TallyState {
    pub fn to_bytes(&self) -> Vec<u8> {
        group_elements_to_bytes(&self.r2s)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        group_elements_from_bytes(bytes).map(|r2s| Self { r2s })
    }
}

fn group_elements_to_bytes(elements: &[gang::GroupElement]) -> Vec<u8> {
    use std::io::Write;
    let mut bytes: Vec<u8> = Vec::with_capacity(GROUP_ELEMENT_BYTES_LEN * elements.len());
    for element in elements {
        bytes.write_all(element.to_bytes().as_ref()).unwrap();
    }
    bytes
}

fn group_elements_from_bytes(bytes: &[u8]) -> Option<Vec<gang::GroupElement>> {
    if bytes.len() % GROUP_ELEMENT_BYTES_LEN != 0 {
        return None;
    }
    let elements = bytes
        .chunks(GROUP_ELEMENT_BYTES_LEN)
        .map(gang::GroupElement::from_bytes)
        .collect::<Option<Vec<_>>>()?;
    Some(elements)
}

#[allow(clippy::ptr_arg)]
pub fn result(
    max_votes: u64,
    table_size: usize,
    tally_state: &TallyState,
    decrypt_shares: &[TallyDecryptShare],
) -> TallyResult {
    let options = tally_state.r2s.len();
    let ris =
        (0..options).map(|i| gang::GroupElement::sum(decrypt_shares.iter().map(|ds| &ds.r1s[i])));

    let mut r_results = tally_state
        .r2s
        .iter()
        .zip(ris)
        .map(|(r2, r1)| r2 - r1)
        .collect::<Vec<_>>();
    for r in r_results.iter_mut() {
        r.normalize()
    }

    let mut votes = Vec::new();
    let mut vote_left = max_votes;

    let table = gang::GroupElement::table(table_size);
    for r in r_results {
        let mut found = None;

        if r == gang::GroupElement::zero() {
            found = Some(0)
        } else {
            for (ith, table_elem) in table.iter().enumerate() {
                if table_elem == &r {
                    found = Some(ith as u64 + 1);
                    break;
                }
            }

            if found.is_none() {
                let gen = gang::GroupElement::generator();
                let mut e = &table[table_size - 1] + &gen;
                let mut i = table_size as u64 + 1;
                loop {
                    if i >= vote_left {
                        break;
                    }

                    if e == r {
                        found = Some(i);
                        break;
                    }
                    e = &e + &gen;
                    i += 1;
                }
            }
        }

        match found {
            None => votes.push(None),
            Some(votes_found) => {
                vote_left -= votes_found;
                votes.push(Some(votes_found))
            }
        }
    }
    TallyResult { votes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    #[test]
    fn encdec1() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let h = CRS::random(&mut rng);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let participants = vec![m1.public_key()];
        let ek = EncryptingVoteKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;
        let e1 = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 0));
        let e2 = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 1));
        let e3 = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 0));

        println!("tallying");

        let mut tally = Tally::new(vote_options);
        tally.add(&e1.0, 6);
        tally.add(&e2.0, 5);
        tally.add(&e3.0, 4);

        let (ts, tds1) = tally.finish(m1.secret_key());

        let max_votes = 20;

        let shares = vec![tds1];

        println!("resulting");
        let tr = result(max_votes, 5, &ts, &shares);

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], Some(10), "vote for option 0");
        assert_eq!(tr.votes[1], Some(5), "vote for option 1");
    }

    #[test]
    fn encdec3() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let h = CRS::random(&mut rng);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc2 = MemberCommunicationKey::new(&mut rng);
        let mc3 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public(), mc2.to_public(), mc3.to_public()];

        let threshold = 3;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);
        let m2 = MemberState::new(&mut rng, threshold, &h, &mc, 1);
        let m3 = MemberState::new(&mut rng, threshold, &h, &mc, 2);

        let participants = vec![m1.public_key(), m2.public_key(), m3.public_key()];
        let ek = EncryptingVoteKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;
        let (e1, e1_proof) = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 0));
        let e2 = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 1));
        let e3 = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 0));

        assert!(verify_vote(&ek, &e1, &e1_proof));
        println!("tallying");

        let mut tally = Tally::new(vote_options);
        tally.add(&e1, 1);
        tally.add(&e2.0, 3);
        tally.add(&e3.0, 4);

        let (_, tds1) = tally.finish(m1.secret_key());
        let (_, tds2) = tally.finish(m2.secret_key());
        let (ts, tds3) = tally.finish(m3.secret_key());

        let max_votes = 20;

        let shares = vec![tds1, tds2, tds3];

        println!("resulting");
        let tr = result(max_votes, 5, &ts, &shares);

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], Some(5), "vote for option 0");
        assert_eq!(tr.votes[1], Some(3), "vote for option 1");
    }
}
