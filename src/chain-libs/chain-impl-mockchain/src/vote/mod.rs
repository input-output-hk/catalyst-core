//! cover all that relate to the voting part of things
//! (except for the certificate that are in the certificate
//! module).
//!

mod choice;
mod committee;
mod ledger;
mod manager;
mod payload;
mod privacy;
mod status;
mod tally;

pub use self::{
    choice::{Choice, Options},
    committee::CommitteeId,
    ledger::{VotePlanLedger, VotePlanLedgerError},
    manager::{ValidatedPayload, VoteError, VotePlanManager},
    payload::{EncryptedVote, Payload, PayloadType, ProofOfCorrectVote, TryFromIntError},
    privacy::encrypt_vote,
    status::{VotePlanStatus, VoteProposalStatus},
    tally::{PrivateTallyState, Tally, TallyError, TallyResult, Weight},
};
