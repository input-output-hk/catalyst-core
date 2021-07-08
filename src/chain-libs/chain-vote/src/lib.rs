#[macro_use]
mod macros;
pub mod committee;
mod cryptography;
mod encrypted_vote;
mod math;
pub mod tally;

// re-export under a debug module
#[doc(hidden)]
pub mod debug {
    pub mod cryptography {
        pub use crate::cryptography::*;
    }
}

#[cfg(crypto_backend = "__internal_ex_backend_p256k1")]
pub(crate) use chain_crypto::ec::p256k1::*;
#[cfg(crypto_backend = "__internal_ex_backend_ristretto255")]
pub(crate) use chain_crypto::ec::ristretto255::*;

pub use math::babystep::BabyStepsTable as TallyOptimizationTable;

pub use crate::{
    committee::{ElectionPublicKey, MemberCommunicationKey, MemberPublicKey, MemberState},
    cryptography::Ciphertext, //todo: why this?
    encrypted_vote::{Ballot, BallotVerificationError, EncryptedVote, ProofOfCorrectVote, Vote},
    tally::{Crs, EncryptedTally, Tally, TallyDecryptShare},
};
