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

pub use chain_crypto::ec::BabyStepsTable as TallyOptimizationTable;

pub use crate::{
    committee::{ElectionPublicKey, MemberCommunicationKey, MemberPublicKey, MemberState},
    cryptography::Ciphertext, //todo: why this?
    encrypted_vote::{Ballot, BallotVerificationError, EncryptedVote, ProofOfCorrectVote, Vote},
    tally::{Crs, EncryptedTally, Tally, TallyDecryptShare},
};
