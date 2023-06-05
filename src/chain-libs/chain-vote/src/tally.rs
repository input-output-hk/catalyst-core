use std::num::NonZeroU64;

use crate::GroupElement;
use crate::{
    committee::*,
    cryptography::{Ciphertext, CorrectShareGenerationZkp},
    encrypted_vote::Ballot,
    math::babystep::baby_step_giant_step,
    TallyOptimizationTable,
};
use base64::{engine::general_purpose, Engine as _};
use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;
use rand_core::{CryptoRng, RngCore};

/// Secret key for opening vote
pub type OpeningVoteKey = MemberSecretKey;

/// Submitted vote, which constists of an `EncryptedVote` and a `
/// Common Reference String
pub type Crs = GroupElement;

/// An encrypted vote is only valid for specific values of the election public key and crs.
/// It may be useful to check early if a vote is valid before actually adding it to the tally,
/// and it is therefore important to verify that it is later added to an encrypted tally that
/// is consistent with the election public key and crs it was verified against.
/// To reduce memory occupation, we use a hash of those two values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ElectionFingerprint([u8; ElectionFingerprint::BYTES_LEN]);

impl ElectionFingerprint {
    const BYTES_LEN: usize = 32;
}

impl From<(&ElectionPublicKey, &Crs)> for ElectionFingerprint {
    fn from(from: (&ElectionPublicKey, &Crs)) -> Self {
        let (election_pk, crs) = from;
        let mut hasher = Blake2b::new(32);
        hasher.input(&crs.to_bytes());
        hasher.input(&election_pk.to_bytes());
        let mut fingerprint = [0; 32];
        hasher.result(&mut fingerprint);
        Self(fingerprint)
    }
}

/// `EncryptedTally` is formed by one ciphertext per existing option and a fingerprint that
/// identifies the election parameters used (crs and election public key)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncryptedTally {
    r: Vec<Ciphertext>,
    fingerprint: ElectionFingerprint,
    max_stake: u64,
}

/// `TallyDecryptShare` contains one decryption share per existing option. All committee
/// members (todo: this will change once DKG is completed)
/// need to submit a `TallyDecryptShare` in order to successfully decrypt
/// the `EncryptedTally`.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TallyDecryptShare {
    elements: Vec<ProvenDecryptShare>,
}

/// `ValidatedTally` can only be constructed by valid `TallyDecryptShare`s, and the
/// corresponding `EncryptedTally`. This intermediate structure ensures that only
/// validated decryptions are used to compute the election outcome, i.e. if the
/// committee members do not present valid shares, the tally decryption cannot be
/// computed.
/// This intermediate structure is particularly of interest during the distributed
/// decryption protocol, where, in case there is a misbehaving party, one needs to
/// perform certain actions between the verification of a decryption share, and its
/// use in the decrypted tally computation.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ValidatedTally {
    r: Vec<Ciphertext>,
    decrypt_shares: Vec<TallyDecryptShare>,
    max_stake: u64,
}

/// `ProvenDecryptShare` consists of a group element (the partial decryption), and `CorrectShareGenerationZkp`,
/// a proof of correct decryption.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct ProvenDecryptShare {
    r1: GroupElement,
    pi: CorrectShareGenerationZkp,
}

/// `Tally` represents the decrypted tally, with one `u64` result for each of the options of the
/// election.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Tally {
    pub votes: Vec<u64>,
}

#[derive(Debug, thiserror::Error)]
#[error("invalid data for private tally")]
pub struct TallyError;

#[derive(Debug, thiserror::Error)]
#[error("Incorrect decryption shares")]
pub struct DecryptionError;

#[derive(Debug, thiserror::Error)]
#[error("Incorrect decryption shares")]
pub struct EncryptedTallyError;

/// Base64 decode error
#[derive(Debug, thiserror::Error)]
#[error("Invalid data, cannot decode base64 {0}")]
pub struct Base64DecodeError(String);

impl EncryptedTally {
    const MAX_STAKE_BYTES_LEN: usize = std::mem::size_of::<u64>();

    /// Initialise a new tally with N different options. The `EncryptedTally` is computed using
    /// the additive homomorphic property of the elgamal `Ciphertext`s, and is therefore initialised
    /// with zero ciphertexts.
    pub fn new(options: usize, election_pk: ElectionPublicKey, crs: Crs) -> Self {
        let r = vec![Ciphertext::zero(); options];
        EncryptedTally {
            r,
            fingerprint: (&election_pk, &crs).into(),
            max_stake: 0,
        }
    }

    /// Returns base64 representation of `EncryptedTally`
    #[must_use]
    pub fn to_base64(&self) -> String {
        let bytes = self.to_bytes();
        general_purpose::STANDARD.encode(bytes)
    }

    /// Generate `EncryptedTally` type from base64 representation
    /// # Errors
    /// - `Base64DecodeError`
    pub fn from_base_64(encrypted_tally_b64: String) -> Result<Self, Box<dyn std::error::Error>> {
        match general_purpose::STANDARD.decode(encrypted_tally_b64) {
            Ok(bytes) => match EncryptedTally::from_bytes(&bytes) {
                Some(encrypted_tally) => Ok(encrypted_tally),
                None => Err(Box::new(EncryptedTallyError)),
            },
            Err(err) => Err(Box::new(Base64DecodeError(err.to_string()))),
        }
    }

    /// Add a submitted `ballot`, with a specific `weight` to the tally.
    /// Remember that a vote is only valid for a specific election (i.e. pair of
    /// election public key and crs), and trying to add a ballot validated for a
    /// different one will result in a panic.
    ///
    /// Note that the encrypted vote needs to have the exact same number of
    /// options as the initialised tally, otherwise an assert will trigger.
    pub fn add(&mut self, ballot: &Ballot, weight: u64) {
        assert_eq!(ballot.vote().len(), self.r.len());
        assert_eq!(ballot.fingerprint(), &self.fingerprint);
        for (ri, ci) in self.r.iter_mut().zip(ballot.vote().iter()) {
            *ri = &*ri + &(ci * weight);
        }
        self.max_stake += weight;
    }

    /// Given a single committee member's `secret_key`, returns a partial decryption of
    /// the `EncryptedTally`
    pub fn partial_decrypt<R: RngCore + CryptoRng>(
        &self,
        rng: &mut R,
        secret_key: &OpeningVoteKey,
    ) -> TallyDecryptShare {
        let mut dshares = Vec::with_capacity(self.r.len());
        let mut r2s = Vec::with_capacity(self.r.len());
        for r in &self.r {
            // todo: we are decrypting twice, we can probably improve this
            let decrypted_share = &r.e1 * &secret_key.0.sk;
            let proof = CorrectShareGenerationZkp::generate(
                r,
                &secret_key.to_public().0,
                &decrypted_share,
                &secret_key.0,
                rng,
            );
            dshares.push(ProvenDecryptShare {
                r1: decrypted_share,
                pi: proof,
            });
            r2s.push(r.e2.clone());
        }
        TallyDecryptShare { elements: dshares }
    }

    /// Given the members `pks`, and their corresponding `decrypte_shares`, this function validates
    /// the different shares, and returns a `ValidatedTally`, or a `DecryptionError`.
    pub fn validate_partial_decryptions(
        &self,
        pks: &[MemberPublicKey],
        decrypt_shares: &[TallyDecryptShare],
    ) -> Result<ValidatedTally, DecryptionError> {
        for (pk, decrypt_share) in pks.iter().zip(decrypt_shares.iter()) {
            if !decrypt_share.verify(self, pk) {
                return Err(DecryptionError);
            }
        }
        Ok(ValidatedTally {
            r: self.r.clone(),
            decrypt_shares: decrypt_shares.to_vec(),
            max_stake: self.max_stake,
        })
    }

    /// Returns a byte array with every ciphertext in the `EncryptedTally`
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(
            Ciphertext::BYTES_LEN * self.r.len()
                + ElectionFingerprint::BYTES_LEN
                + Self::MAX_STAKE_BYTES_LEN,
        );
        bytes.extend_from_slice(&self.fingerprint.0);
        let max_stake_bytes = self.max_stake.to_le_bytes();
        bytes.extend_from_slice(&max_stake_bytes);
        for ri in &self.r {
            bytes.extend_from_slice(ri.to_bytes().as_ref());
        }
        bytes
    }

    /// Tries to generate an `EncryptedTally` out of an array of bytes. Returns `None` if the
    /// size of the byte array is not a multiply of `Ciphertext::BYTES_LEN`.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let cyphertext_len =
            bytes.len() - ElectionFingerprint::BYTES_LEN - Self::MAX_STAKE_BYTES_LEN;
        if cyphertext_len % Ciphertext::BYTES_LEN != 0 {
            return None;
        }
        let (fingerprint_bytes, bytes) = bytes.split_at(ElectionFingerprint::BYTES_LEN);
        let fingerprint = ElectionFingerprint(fingerprint_bytes.try_into().unwrap());

        let (max_stake_bytes, bytes) = bytes.split_at(Self::MAX_STAKE_BYTES_LEN);
        let max_stake = u64::from_le_bytes(max_stake_bytes.try_into().unwrap());

        let r = bytes
            .chunks(Ciphertext::BYTES_LEN)
            .map(Ciphertext::from_bytes)
            .collect::<Option<Vec<_>>>()?;

        Some(Self {
            r,
            fingerprint,
            max_stake,
        })
    }
}

impl ValidatedTally {
    pub fn len(&self) -> usize {
        self.r.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // Given the shares of the committee members, returns the decryption of all the
    // election options in the form of `GroupElements`. To get the final results, one
    // needs to compute the discrete logarithm of these values, which is performed in
    // `decrypt_tally`.
    fn decrypt(&self) -> Vec<GroupElement> {
        let state: Vec<GroupElement> = self.r.iter().map(|c| c.e2.clone()).collect();
        let ris = (0..state.len())
            .map(|i| GroupElement::sum(self.decrypt_shares.iter().map(|ds| &ds.elements[i].r1)));

        state
            .iter()
            .zip(ris)
            .map(|(r2, r1)| r2 - r1)
            .collect::<Vec<_>>()
    }

    /// Given the `decrypt_shares` of all committee members, `max_votes`, and a tally optimization
    /// table, `decrypt_tally` first decrypts `self`, and then computes the discrete logarithm
    /// of each resulting plaintext.
    pub fn decrypt_tally(&self, table: &TallyOptimizationTable) -> Result<Tally, TallyError> {
        let r_results = self.decrypt();
        let votes =
            baby_step_giant_step(r_results, self.max_stake, table).map_err(|_| TallyError)?;
        Ok(Tally { votes })
    }
}

impl std::ops::Add for EncryptedTally {
    type Output = Self;

    // Ads two `EncryptedTally`, leveraging the additive homomorphic property of the
    // underlying ciphertexts. If the public keys or the crs are not equal, it panics
    // todo: maybe we want to handle the errors?
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.r.len(), rhs.r.len());
        let r = self
            .r
            .iter()
            .zip(rhs.r.iter())
            .map(|(left, right)| left + right)
            .collect();
        let max_stake = self.max_stake + rhs.max_stake;
        Self {
            r,
            fingerprint: self.fingerprint,
            max_stake,
        }
    }
}

impl ProvenDecryptShare {
    const SIZE: usize = CorrectShareGenerationZkp::PROOF_SIZE + GroupElement::BYTES_LEN;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != ProvenDecryptShare::SIZE {
            return None;
        }

        let r1 = GroupElement::from_bytes(&bytes[0..GroupElement::BYTES_LEN])?;
        let proof = CorrectShareGenerationZkp::from_bytes(&bytes[GroupElement::BYTES_LEN..])?;
        Some(ProvenDecryptShare { r1, pi: proof })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut output = [0u8; Self::SIZE];
        output[0..GroupElement::BYTES_LEN].copy_from_slice(&self.r1.to_bytes());
        output[GroupElement::BYTES_LEN..].copy_from_slice(&self.pi.to_bytes());
        output
    }
}

impl TallyDecryptShare {
    /// Given the member's public key `MemberPublicKey`, and the `EncryptedTally`, verifies the
    /// correctness of the `TallyDecryptShare`.
    pub fn verify(&self, encrypted_tally: &EncryptedTally, pk: &MemberPublicKey) -> bool {
        // elements amount of the tally share should be equal to the number of the existing options of the tally
        if self.options() != encrypted_tally.r.len() {
            return false;
        }

        for (element, r) in self.elements.iter().zip(encrypted_tally.r.iter()) {
            if !element.pi.verify(r, &element.r1, &pk.0) {
                return false;
            }
        }
        true
    }

    /// Number of voting options this tally decrypt share structure is
    /// constructed for.
    pub fn options(&self) -> usize {
        self.elements.len()
    }

    /// Size of the byte representation for a tally decrypt share
    /// with the given number of options.
    pub fn bytes_len(options: usize) -> usize {
        (CorrectShareGenerationZkp::PROOF_SIZE + GroupElement::BYTES_LEN)
            .checked_mul(options)
            .expect("integer overflow")
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for element in self.elements.iter() {
            out.extend_from_slice(&element.to_bytes());
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

impl Tally {
    /// Verifies that `TallyDecryptShare` are correct decryptions of `encrypted_tally` for public
    /// keys `pks`.
    ///
    /// Verifies that the decrypted tally was correctly obtained from the given
    /// `EncryptedTally` and `TallyDecryptShare` parts.
    ///
    /// This can be used for quick online validation for the tallying
    /// performed offline.
    pub fn verify(
        &self,
        encrypted_tally: &EncryptedTally,
        pks: &[MemberPublicKey],
        decrypt_shares: &[TallyDecryptShare],
    ) -> bool {
        let validated_decryptions =
            match encrypted_tally.validate_partial_decryptions(pks, decrypt_shares) {
                Ok(dec) => dec,
                Err(_) => return false,
            };

        let r_results = validated_decryptions.decrypt();
        let gen = GroupElement::generator();
        for (i, &w) in self.votes.iter().enumerate() {
            if &gen * w != r_results[i] {
                return false;
            }
        }
        true
    }
}

/// Decrypt a slice of `ValidatedTally`s, using a single baby-step giant-step table
pub fn batch_decrypt(
    validated_tallies: impl AsRef<[ValidatedTally]>,
) -> Result<Vec<Tally>, TallyError> {
    let validated_tallies = validated_tallies.as_ref();
    let absolute_max_stake = validated_tallies.iter().map(|tally| tally.max_stake).max();

    let absolute_max_stake = match absolute_max_stake {
        Some(stake) => stake,
        None => return Ok(vec![]),
    };

    match NonZeroU64::try_from(absolute_max_stake) {
        // if absolute_max_stake == 0, all stakes are zero, so safe to do a trivial conversion
        Err(_) => Ok(validated_tallies.iter().map(trivial_convert).collect()),
        Ok(absolute_max_stake) => {
            let table = TallyOptimizationTable::generate(absolute_max_stake);
            validated_tallies
                .iter()
                .map(|tally| tally.decrypt_tally(&table))
                .collect()
        }
    }
}

/// Convert from [`ValidatedTally`] to a [`Tally`] when there are no votes cast
///
/// If `max_stake` is not 0, this function will panic
fn trivial_convert(validated_tally: &ValidatedTally) -> Tally {
    assert_eq!(validated_tally.max_stake, 0);
    let votes = vec![0; validated_tally.len()];
    Tally { votes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cryptography::{Keypair, PublicKey};
    use crate::encrypted_vote::Vote;
    use crate::Ballot;
    use rand_chacha::ChaCha20Rng;
    use rand_core::{CryptoRng, RngCore, SeedableRng};

    fn get_encrypted_ballot<R: RngCore + CryptoRng>(
        rng: &mut R,
        pk: &ElectionPublicKey,
        crs: &Crs,
        vote: Vote,
    ) -> Ballot {
        let (enc, proof) = pk.encrypt_and_prove_vote(rng, crs, vote);
        Ballot::try_from_vote_and_proof(enc, &proof, crs, pk).unwrap()
    }

    #[test]
    fn encdec1() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&shared_string);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let participants = vec![m1.public_key()];
        let ek = ElectionPublicKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;
        let e1 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());
        let e2 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 1).unwrap());
        let e3 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());

        println!("tallying");

        let mut encrypted_tally = EncryptedTally::new(vote_options, ek.clone(), h.clone());
        encrypted_tally.add(&e1, 6);
        encrypted_tally.add(&e2, 5);
        encrypted_tally.add(&e3, 4);

        let tds1 = encrypted_tally.partial_decrypt(&mut rng, m1.secret_key());

        let max_votes = 20;

        let shares = vec![tds1];

        println!("resulting");
        let table = TallyOptimizationTable::generate_with_balance(
            max_votes.try_into().unwrap(),
            1.try_into().unwrap(),
        );
        let tr = encrypted_tally
            .validate_partial_decryptions(&participants, &shares)
            .unwrap()
            .decrypt_tally(&table)
            .unwrap();

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], 10, "vote for option 0");
        assert_eq!(tr.votes[1], 5, "vote for option 1");

        println!("verifying");
        assert!(tr.verify(&encrypted_tally, &participants, &shares));
    }

    #[test]
    fn encdec3() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&shared_string);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc2 = MemberCommunicationKey::new(&mut rng);
        let mc3 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public(), mc2.to_public(), mc3.to_public()];

        let threshold = 3;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);
        let m2 = MemberState::new(&mut rng, threshold, &h, &mc, 1);
        let m3 = MemberState::new(&mut rng, threshold, &h, &mc, 2);

        let participants = vec![m1.public_key(), m2.public_key(), m3.public_key()];
        let ek = ElectionPublicKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;
        let e1 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());
        let e2 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 1).unwrap());
        let e3 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());

        println!("tallying");

        let mut encrypted_tally = EncryptedTally::new(vote_options, ek, h);
        encrypted_tally.add(&e1, 1);
        encrypted_tally.add(&e2, 3);
        encrypted_tally.add(&e3, 4);

        let tds1 = encrypted_tally.partial_decrypt(&mut rng, m1.secret_key());
        let tds2 = encrypted_tally.partial_decrypt(&mut rng, m2.secret_key());
        let tds3 = encrypted_tally.partial_decrypt(&mut rng, m3.secret_key());

        // check a mismatch parameters (m2 key with m1's share) is detected
        assert!(!tds1.verify(&encrypted_tally, &m2.public_key()));

        let max_votes = 20;

        let shares = vec![tds1, tds2, tds3];

        println!("resulting");
        let table = TallyOptimizationTable::generate_with_balance(
            max_votes.try_into().unwrap(),
            1.try_into().unwrap(),
        );
        let tr = encrypted_tally
            .validate_partial_decryptions(&participants, &shares)
            .unwrap()
            .decrypt_tally(&table)
            .unwrap();

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], 5, "vote for option 0");
        assert_eq!(tr.votes[1], 3, "vote for option 1");

        println!("verifying");
        assert!(tr.verify(&encrypted_tally, &participants, &shares));
    }

    #[test]
    fn zero_and_max_votes() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&shared_string);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let participants = vec![m1.public_key()];
        let ek = ElectionPublicKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;

        println!("tallying");

        let mut encrypted_tally = EncryptedTally::new(vote_options, ek.clone(), h.clone());
        encrypted_tally.add(
            &get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap()),
            42,
        );

        let tds1 = encrypted_tally.partial_decrypt(&mut rng, m1.secret_key());

        let max_votes = 42;

        let shares = vec![tds1];

        println!("resulting");
        let table = TallyOptimizationTable::generate_with_balance(
            max_votes.try_into().unwrap(),
            1.try_into().unwrap(),
        );
        let tr = encrypted_tally
            .validate_partial_decryptions(&participants, &shares)
            .unwrap()
            .decrypt_tally(&table)
            .unwrap();

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], 42, "vote for option 0");
        assert_eq!(tr.votes[1], 0, "vote for option 1");

        println!("verifying");
        assert!(tr.verify(&encrypted_tally, &participants, &shares));
    }

    #[test]
    fn empty_tally() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&shared_string);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public()];

        let threshold = 1;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let vote_options = 2;

        println!("tallying");

        let encrypted_tally = EncryptedTally::new(
            vote_options,
            ElectionPublicKey::from_participants(&[m1.public_key()]),
            h,
        );
        let tds1 = encrypted_tally.partial_decrypt(&mut rng, m1.secret_key());

        let max_votes = 2;

        let shares = vec![tds1];

        println!("resulting");
        let table = TallyOptimizationTable::generate_with_balance(
            max_votes.try_into().unwrap(),
            1.try_into().unwrap(),
        );
        let tr = encrypted_tally
            .validate_partial_decryptions(&[m1.public_key()], &shares)
            .unwrap()
            .decrypt_tally(&table)
            .unwrap();

        println!("{:?}", tr);
        assert_eq!(tr.votes.len(), vote_options);
        assert_eq!(tr.votes[0], 0, "vote for option 0");
        assert_eq!(tr.votes[1], 0, "vote for option 1");

        println!("verifying");
        assert!(tr.verify(&encrypted_tally, &[m1.public_key()], &shares));
    }

    #[test]
    fn tally_wrong_elements_size() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&shared_string);

        let mc = MemberCommunicationKey::new(&mut rng);
        let mc = [mc.to_public()];

        let threshold = 1;

        let m = MemberState::new(&mut rng, threshold, &h, &mc, 0);

        let participants = vec![m.public_key()];
        let ek = ElectionPublicKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;
        let e = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());

        println!("tallying");

        let mut encrypted_tally = EncryptedTally::new(vote_options, ek, h);
        encrypted_tally.add(&e, 1);

        let mut tds = encrypted_tally.partial_decrypt(&mut rng, m.secret_key());

        println!("corrupt tally share, add extra element");

        tds.elements.push(tds.elements.last().unwrap().clone());

        assert!(!tds.verify(&encrypted_tally, &m.public_key()))
    }

    #[test]
    fn zero_encrypted_tally_serialization_sanity() {
        let election_key = ElectionPublicKey(PublicKey {
            pk: GroupElement::from_hash(&[1u8]),
        });
        let h = Crs::from_hash(&[1u8]);
        let tally = EncryptedTally::new(3, election_key, h);
        let bytes = tally.to_bytes();
        dbg!(bytes.len());
        let deserialized_tally = EncryptedTally::from_bytes(&bytes).unwrap();
        assert_eq!(tally, deserialized_tally);
    }

    #[test]
    fn serialize_tally_decrypt_share() {
        let mut r = ChaCha20Rng::from_seed([0u8; 32]);

        let keypair = Keypair::generate(&mut r);

        let plaintext = GroupElement::from_hash(&[0u8]);
        let ciphertext = keypair.public_key.encrypt_point(&plaintext, &mut r);

        let share = &ciphertext.e1 * &keypair.secret_key.sk;

        let proof = CorrectShareGenerationZkp::generate(
            &ciphertext,
            &keypair.public_key,
            &share,
            &keypair.secret_key,
            &mut r,
        );

        let prover_share = ProvenDecryptShare {
            pi: proof,
            r1: share,
        };
        let tally_dec_share = TallyDecryptShare {
            elements: vec![prover_share],
        };
        let bytes = tally_dec_share.to_bytes();

        assert_eq!(bytes.len(), TallyDecryptShare::bytes_len(1));

        let tally_from_bytes = TallyDecryptShare::from_bytes(&bytes);
        assert!(tally_from_bytes.is_some());
    }

    #[test]
    fn batch_decrypt_empty_slice() {
        assert_eq!(batch_decrypt(&[]).unwrap(), []);
    }

    #[test]
    fn batch_decrypt_with_data() {
        let mut rng = ChaCha20Rng::from_seed([0u8; 32]);

        let shared_string =
            b"Example of a shared string. This should be VotePlan.to_id()".to_owned();
        let h = Crs::from_hash(&shared_string);

        let mc1 = MemberCommunicationKey::new(&mut rng);
        let mc2 = MemberCommunicationKey::new(&mut rng);
        let mc3 = MemberCommunicationKey::new(&mut rng);
        let mc = [mc1.to_public(), mc2.to_public(), mc3.to_public()];

        let threshold = 3;

        let m1 = MemberState::new(&mut rng, threshold, &h, &mc, 0);
        let m2 = MemberState::new(&mut rng, threshold, &h, &mc, 1);
        let m3 = MemberState::new(&mut rng, threshold, &h, &mc, 2);

        let participants = vec![m1.public_key(), m2.public_key(), m3.public_key()];
        let ek = ElectionPublicKey::from_participants(&participants);

        println!("encrypting vote");

        let vote_options = 2;
        let e1 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());
        let e2 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 1).unwrap());
        let e3 = get_encrypted_ballot(&mut rng, &ek, &h, Vote::new(vote_options, 0).unwrap());

        println!("tallying");

        let mut encrypted_tally = EncryptedTally::new(vote_options, ek, h);
        encrypted_tally.add(&e1, 1);
        encrypted_tally.add(&e2, 3);
        encrypted_tally.add(&e3, 4);

        let tds1 = encrypted_tally.partial_decrypt(&mut rng, m1.secret_key());
        let tds2 = encrypted_tally.partial_decrypt(&mut rng, m2.secret_key());
        let tds3 = encrypted_tally.partial_decrypt(&mut rng, m3.secret_key());

        // check a mismatch parameters (m2 key with m1's share) is detected
        assert!(!tds1.verify(&encrypted_tally, &m2.public_key()));

        let shares = vec![tds1, tds2, tds3];

        println!("resulting");
        let validated_tally = encrypted_tally
            .validate_partial_decryptions(&participants, &shares)
            .unwrap();

        let tallies = batch_decrypt([validated_tally]).unwrap();

        assert_eq!(tallies.len(), 1);
        assert_eq!(tallies[0].votes, vec![5, 3]);
    }
}
