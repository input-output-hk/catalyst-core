use crate::vote::{EncryptedVote, ProofOfCorrectVote};
use chain_vote::{Crs, ElectionPublicKey, Vote};
use rand_core::{CryptoRng, RngCore};

#[allow(dead_code)]
pub fn encrypt_vote<R: RngCore + CryptoRng>(
    rng: &mut R,
    crs: &Crs,
    public_key: &ElectionPublicKey,
    vote: Vote,
) -> (EncryptedVote, ProofOfCorrectVote) {
    let (ev, proof) = public_key.encrypt_and_prove_vote(rng, crs, vote);
    (
        EncryptedVote::from_inner(ev),
        ProofOfCorrectVote::from_inner(proof),
    )
}
