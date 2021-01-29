//! Chain Vote

mod commitment;
pub mod committee;
mod encrypted;
mod gang;
pub mod gargamel;
mod hybrid;
mod math;
pub mod shvzk;
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

pub use committee::{
    MemberCommunicationKey, MemberCommunicationPublicKey, MemberPublicKey, MemberState,
};
pub use encrypted::EncryptingVote;
use gang::GroupElement;
pub use gang::{BabyStepsTable as TallyOptimizationTable, Scalar};
pub use gargamel::Ciphertext;
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptedTally {
    r: Vec<Ciphertext>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TallyDecryptShare {
    r1s: Vec<gang::GroupElement>,
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

impl TallyDecryptShare {
    /// Number of voting options this taly decrypt share structure is
    /// constructed for.
    pub fn options(&self) -> usize {
        self.r1s.len()
    }

    /// Size of the byte representation for a tally decrypt share
    /// with the given number of options.
    pub fn bytes_len(options: usize) -> usize {
        group_elements_bytes_len(options)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        group_elements_to_bytes(&self.r1s)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        group_elements_from_bytes(bytes).map(|r1s| Self { r1s })
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

fn result_vector(
    tally_state: &TallyState,
    decrypt_shares: &[TallyDecryptShare],
) -> Vec<gang::GroupElement> {
    let ris = (0..tally_state.r2s.len())
        .map(|i| gang::GroupElement::sum(decrypt_shares.iter().map(|ds| &ds.r1s[i])));

    let mut results = tally_state
        .r2s
        .iter()
        .zip(ris)
        .map(|(r2, r1)| r2 - r1)
        .collect::<Vec<_>>();
    for r in results.iter_mut() {
        r.normalize()
    }

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
            if &gen * gang::Scalar::from_u64(w) != r_results[i] {
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

        let mut tally = EncryptedTally::new(vote_options);
        tally.add(&e1.0, 6);
        tally.add(&e2.0, 5);
        tally.add(&e3.0, 4);

        let (ts, tds1) = tally.finish(m1.secret_key());

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

        let mut tally = EncryptedTally::new(vote_options);
        tally.add(&e1, 1);
        tally.add(&e2.0, 3);
        tally.add(&e3.0, 4);

        let (_, tds1) = tally.finish(m1.secret_key());
        let (_, tds2) = tally.finish(m2.secret_key());
        let (ts, tds3) = tally.finish(m3.secret_key());

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

        let h = CRS::random(&mut rng);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let participants = vec![m1.public_key()];
        let ek = EncryptingVoteKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;
        let (e1, _) = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 0));

        println!("tallying");

        let mut tally = EncryptedTally::new(vote_options);
        tally.add(&e1, 42);

        let (ts, tds1) = tally.finish(m1.secret_key());

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

        let h = CRS::random(&mut rng);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let vote_options = 2;

        println!("tallying");

        let tally = EncryptedTally::new(vote_options);
        let (ts, tds1) = tally.finish(m1.secret_key());

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
        let e1 = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 0));
        let e2 = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 1));
        let e3 = encrypt_vote(&mut rng, &ek, Vote::new(vote_options, 0));

        let mut tally = EncryptedTally::new(vote_options);
        tally.add(&e1.0, 10);
        tally.add(&e2.0, 3);
        tally.add(&e3.0, 40);

        let (_, tds1) = tally.finish(m1.secret_key());
        let (_, tds2) = tally.finish(m2.secret_key());
        let (ts, tds3) = tally.finish(m3.secret_key());

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
}
