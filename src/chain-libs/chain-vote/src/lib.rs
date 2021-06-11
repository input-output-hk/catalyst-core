//! Chain Vote

#[macro_use]
mod macros;
mod commitment;
pub mod committee;
pub mod decr_nizk;
mod encrypted;
pub mod encryption;
mod gang;
mod math;
pub mod private_voting;
mod unit_vector;

// re-export under a debug module
#[doc(hidden)]
pub mod debug {
    pub mod gang {
        pub use crate::gang::*;
    }
    pub mod private_voting {
        pub use crate::private_voting::*;
    }
}

use decr_nizk::ProofDecrypt;

pub use committee::{
    MemberCommunicationKey, MemberCommunicationPublicKey, MemberPublicKey, MemberState,
};
pub use encrypted::EncryptingVote;
pub use encryption::Ciphertext;
use gang::GroupElement;
pub use gang::{BabyStepsTable as TallyOptimizationTable, Scalar};
use rand_core::{CryptoRng, RngCore};
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

pub type ProofOfCorrectVote = private_voting::unit_vector_zkp::Proof;

/// Common Reference String
pub type Crs = GroupElement;

/// Take a vote and encrypt it + provide a proof of correct voting
pub fn encrypt_vote<R: RngCore + CryptoRng>(
    rng: &mut R,
    crs: &Crs,
    public_key: &EncryptingVoteKey,
    vote: Vote,
) -> (EncryptedVote, ProofOfCorrectVote) {
    let ev = EncryptingVote::prepare(rng, &public_key.0, &vote);
    let proof = private_voting::Proof::generate(rng, &crs, &public_key.0, ev.clone());
    (ev.ciphertexts, proof)
}

/// Verify that the encrypted vote is valid without opening it
#[allow(clippy::ptr_arg)]
pub fn verify_vote(
    crs: &Crs,
    public_key: &EncryptingVoteKey,
    vote: &EncryptedVote,
    proof: &ProofOfCorrectVote,
) -> bool {
    proof.verify(&crs, &public_key.0, vote)
}

/// The encrypted tally
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptedTally {
    r: Vec<Ciphertext>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TallyDecryptShare {
    elements: Vec<ProvenDecryptShare>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProvenDecryptShare {
    r1: gang::GroupElement,
    pi: ProofDecrypt,
}

#[derive(Clone)]
pub struct TallyState {
    r2s: Vec<gang::GroupElement>,
}

/// Decrypted tally with votes indexed per option.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Tally {
    pub votes: Vec<u64>,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid data for private tally")]
pub struct TallyError;

impl EncryptedTally {
    /// Start a new tally with N different options
    pub fn new(options: usize) -> Self {
        let r = vec![Ciphertext::zero(); options];
        EncryptedTally { r }
    }

    /// Add an encrypted vote with a specific weight to the tally
    ///
    /// Note that the encrypted vote needs to have the exact same number of
    /// options as the tally expect otherwise an assert will trigger
    #[allow(clippy::ptr_arg)]
    pub fn add(&mut self, vote: &EncryptedVote, weight: u64) {
        assert_eq!(vote.len(), self.r.len());
        for (ri, ci) in self.r.iter_mut().zip(vote.iter()) {
            *ri = &*ri + &(ci * weight);
        }
    }

    pub fn finish<R: RngCore + CryptoRng>(
        &self,
        rng: &mut R,
        secret_key: &OpeningVoteKey,
    ) -> (TallyState, TallyDecryptShare) {
        let mut dshares = Vec::with_capacity(self.r.len());
        let mut r2s = Vec::with_capacity(self.r.len());
        for r in &self.r {
            // todo: we are decrypting twice, we can probably improve this
            let decrypted_share = &r.e1 * &secret_key.0.sk;
            let pk = MemberPublicKey::from(secret_key);
            let proof = ProofDecrypt::generate(&r, &pk.0, &secret_key.0, rng);
            dshares.push(ProvenDecryptShare {
                r1: decrypted_share,
                pi: proof,
            });
            r2s.push(r.e2.clone());
        }
        (TallyState { r2s }, TallyDecryptShare { elements: dshares })
    }

    pub fn state(&self) -> TallyState {
        TallyState {
            r2s: self.r.iter().map(|r| r.elements().1.clone()).collect(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        use std::io::Write;
        let mut bytes: Vec<u8> = Vec::with_capacity(Ciphertext::BYTES_LEN * self.r.len());
        for ri in &self.r {
            bytes.write_all(ri.to_bytes().as_ref()).unwrap();
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() % Ciphertext::BYTES_LEN != 0 {
            return None;
        }
        let r = bytes
            .chunks(Ciphertext::BYTES_LEN)
            .map(Ciphertext::from_bytes)
            .collect::<Option<Vec<_>>>()?;
        Some(Self { r })
    }
}

impl std::ops::Add for EncryptedTally {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.r.len(), rhs.r.len());
        let r = self
            .r
            .iter()
            .zip(rhs.r.iter())
            .map(|(left, right)| left + right)
            .collect();
        Self { r }
    }
}

impl ProvenDecryptShare {
    const SIZE: usize = decr_nizk::PROOF_SIZE + GroupElement::BYTES_LEN;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != ProvenDecryptShare::SIZE {
            return None;
        }

        let r1 = gang::GroupElement::from_bytes(&bytes[0..GroupElement::BYTES_LEN])?;
        let proof = decr_nizk::ProofDecrypt::from_slice(&bytes[GroupElement::BYTES_LEN..])?;
        Some(ProvenDecryptShare { r1, pi: proof })
    }
}

impl TallyDecryptShare {
    /// Number of voting options this taly decrypt share structure is
    /// constructed for.
    pub fn options(&self) -> usize {
        self.elements.len()
    }

    /// Size of the byte representation for a tally decrypt share
    /// with the given number of options.
    pub fn bytes_len(options: usize) -> usize {
        group_elements_bytes_len(options)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for element in self.elements.iter() {
            out.extend_from_slice(element.r1.to_bytes().as_ref());
            out.extend_from_slice(&element.pi.to_bytes());
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() % ProvenDecryptShare::SIZE != 0 {
            return None;
        }

        let elements = bytes
            .chunks(ProvenDecryptShare::SIZE)
            .map(ProvenDecryptShare::from_bytes)
            .collect::<Option<Vec<_>>>()?;
        Some(TallyDecryptShare { elements })
    }
}

impl TallyState {
    /// Size of the byte representation for tally state
    /// with the given number of options.
    pub fn bytes_len(options: usize) -> usize {
        group_elements_bytes_len(options)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        group_elements_to_bytes(&self.r2s)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        group_elements_from_bytes(bytes).map(|r2s| Self { r2s })
    }
}

fn group_elements_bytes_len(n: usize) -> usize {
    GroupElement::BYTES_LEN
        .checked_mul(n)
        .expect("integer overflow")
}

fn group_elements_to_bytes(elements: &[gang::GroupElement]) -> Vec<u8> {
    use std::io::Write;
    let mut bytes: Vec<u8> = Vec::with_capacity(group_elements_bytes_len(elements.len()));
    for element in elements {
        bytes.write_all(element.to_bytes().as_ref()).unwrap();
    }
    bytes
}

fn group_elements_from_bytes(bytes: &[u8]) -> Option<Vec<gang::GroupElement>> {
    if bytes.len() % GroupElement::BYTES_LEN != 0 {
        return None;
    }

    let elements = bytes
        .chunks(GroupElement::BYTES_LEN)
        .map(gang::GroupElement::from_bytes)
        .collect::<Option<Vec<_>>>()?;
    Some(elements)
}

pub fn verify_decrypt_share(
    encrypted_tally: &EncryptedTally,
    pk: &committee::MemberPublicKey,
    decrypt_share: &TallyDecryptShare,
) -> bool {
    for (element, r) in decrypt_share.elements.iter().zip(encrypted_tally.r.iter()) {
        if !element.pi.verify(&r, &(&r.e2 - &element.r1), &pk.0) {
            return false;
        }
    }
    true
}

fn result_vector(
    tally_state: &TallyState,
    decrypt_shares: &[TallyDecryptShare],
) -> Vec<gang::GroupElement> {
    let ris = (0..tally_state.r2s.len())
        .map(|i| gang::GroupElement::sum(decrypt_shares.iter().map(|ds| &ds.elements[i].r1)));

    let results = tally_state
        .r2s
        .iter()
        .zip(ris)
        .map(|(r2, r1)| r2 - r1)
        .collect::<Vec<_>>();

    results
}

pub fn tally(
    max_votes: u64,
    tally_state: &TallyState,
    decrypt_shares: &[TallyDecryptShare],
    table: &TallyOptimizationTable,
) -> Result<Tally, TallyError> {
    let r_results = result_vector(tally_state, decrypt_shares);
    let votes = gang::baby_step_giant_step(r_results, max_votes, table).map_err(|_| TallyError)?;
    Ok(Tally { votes })
}

impl Tally {
    /// Verifies that the decrypted tally was correctly obtained from the given
    /// `TallyState` and `TallyDecryptShare` parts.
    ///
    /// This can be used for quick online validation for the tallying
    /// performed offline.
    pub fn verify(&self, tally_state: &TallyState, decrypt_shares: &[TallyDecryptShare]) -> bool {
        let r_results = result_vector(tally_state, decrypt_shares);
        let gen = gang::GroupElement::generator();
        for (i, &w) in self.votes.iter().enumerate() {
            if &gen * w != r_results[i] {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    #[test]
    fn encdec1() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let mut shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&mut shared_string);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let participants = vec![m1.public_key()];
        let ek = EncryptingVoteKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;
        let e1 = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 0));
        let e2 = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 1));
        let e3 = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 0));

        println!("tallying");

        let mut tally = EncryptedTally::new(vote_options);
        tally.add(&e1.0, 6);
        tally.add(&e2.0, 5);
        tally.add(&e3.0, 4);

        let (ts, tds1) = tally.finish(&mut rng, m1.secret_key());

        assert_eq!(verify_decrypt_share(&tally, &m1.public_key(), &tds1), true);

        let max_votes = 20;

        let shares = vec![tds1];

        println!("resulting");
        let table = TallyOptimizationTable::generate_with_balance(max_votes, 1);
        let tr = crate::tally(max_votes, &ts, &shares, &table).unwrap();

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], 10, "vote for option 0");
        assert_eq!(tr.votes[1], 5, "vote for option 1");

        println!("verifying");
        assert!(tr.verify(&ts, &shares));
    }

    #[test]
    fn encdec3() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let mut shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&mut shared_string);

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
        let (e1, e1_proof) = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 0));
        let e2 = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 1));
        let e3 = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 0));

        assert!(verify_vote(&h, &ek, &e1, &e1_proof));
        println!("tallying");

        let mut tally = EncryptedTally::new(vote_options);
        tally.add(&e1, 1);
        tally.add(&e2.0, 3);
        tally.add(&e3.0, 4);

        let (_, tds1) = tally.finish(&mut rng, m1.secret_key());
        let (_, tds2) = tally.finish(&mut rng, m2.secret_key());
        let (ts, tds3) = tally.finish(&mut rng, m3.secret_key());

        // check that the verify shares are correct for each participants
        assert_eq!(verify_decrypt_share(&tally, &m1.public_key(), &tds1), true);
        assert_eq!(verify_decrypt_share(&tally, &m2.public_key(), &tds2), true);
        assert_eq!(verify_decrypt_share(&tally, &m3.public_key(), &tds3), true);

        // check a mismatch parameters (m2 key with m1's share) is detected
        assert_eq!(verify_decrypt_share(&tally, &m2.public_key(), &tds1), false);

        let max_votes = 20;

        let shares = vec![tds1, tds2, tds3];

        println!("resulting");
        let table = TallyOptimizationTable::generate_with_balance(max_votes, 1);
        let tr = crate::tally(max_votes, &ts, &shares, &table).unwrap();

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], 5, "vote for option 0");
        assert_eq!(tr.votes[1], 3, "vote for option 1");

        println!("verifying");
        assert!(tr.verify(&ts, &shares));
    }

    #[test]
    fn zero_and_max_votes() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let mut shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&mut shared_string);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let participants = vec![m1.public_key()];
        let ek = EncryptingVoteKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;
        let (e1, _) = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 0));

        println!("tallying");

        let mut tally = EncryptedTally::new(vote_options);
        tally.add(&e1, 42);

        let (ts, tds1) = tally.finish(&mut rng, m1.secret_key());

        let max_votes = 42;

        let shares = vec![tds1];

        println!("resulting");
        let table = TallyOptimizationTable::generate_with_balance(max_votes, 1);
        let tr = crate::tally(max_votes, &ts, &shares, &table).unwrap();

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], 42, "vote for option 0");
        assert_eq!(tr.votes[1], 0, "vote for option 1");

        println!("verifying");
        assert!(tr.verify(&ts, &shares));
    }

    #[test]
    fn empty_tally() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let mut shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&mut shared_string);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let vote_options = 2;

        println!("tallying");

        let tally = EncryptedTally::new(vote_options);
        let (ts, tds1) = tally.finish(&mut rng, m1.secret_key());

        let max_votes = 2;

        let shares = vec![tds1];

        println!("resulting");
        let table = TallyOptimizationTable::generate_with_balance(max_votes, 1);
        let tr = crate::tally(max_votes, &ts, &shares, &table).unwrap();

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], 0, "vote for option 0");
        assert_eq!(tr.votes[1], 0, "vote for option 1");

        println!("verifying");
        assert!(tr.verify(&ts, &shares));
    }

    #[test]
    fn wrong_max_votes() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let mut shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&mut shared_string);

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
        let e1 = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 0));
        let e2 = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 1));
        let e3 = encrypt_vote(&mut rng, &h, &ek, Vote::new(vote_options, 0));

        let mut tally = EncryptedTally::new(vote_options);
        tally.add(&e1.0, 10);
        tally.add(&e2.0, 3);
        tally.add(&e3.0, 40);

        let (_, tds1) = tally.finish(&mut rng, m1.secret_key());
        let (_, tds2) = tally.finish(&mut rng, m2.secret_key());
        let (ts, tds3) = tally.finish(&mut rng, m3.secret_key());

        let max_votes = 4;

        let shares = vec![tds1, tds2, tds3];

        println!("resulting");
        let table = TallyOptimizationTable::generate_with_balance(max_votes, 1);
        let res = crate::tally(max_votes, &ts, &shares, &table);
        assert!(
            res.is_err(),
            "unexpected successful tally: {:?}",
            res.ok().unwrap()
        );
    }

    #[test]
    fn zero_encrypted_tally_serialization_sanity() {
        let tally = EncryptedTally::new(3);
        let bytes = tally.to_bytes();
        let deserialized_tally = EncryptedTally::from_bytes(&bytes).unwrap();
        assert_eq!(tally, deserialized_tally);
    }
}
