use crate::vote::ProofOfCorrectVote;
use chain_vote::{EncryptedVote, EncryptingVoteKey, Vote};
use rand_core::{CryptoRng, RngCore};

#[allow(dead_code)]
pub fn encrypt_vote<R: RngCore + CryptoRng>(
    rng: &mut R,
    public_key: &EncryptingVoteKey,
    vote: Vote,
) -> (EncryptedVote, ProofOfCorrectVote) {
    let (ev, proof) = chain_vote::encrypt_vote(rng, public_key, vote);
    (ev, ProofOfCorrectVote::from_inner(proof))
}
