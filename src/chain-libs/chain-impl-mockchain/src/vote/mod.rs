//! cover all that relate to the voting part of things
//! (except for the certificate that are in the certificate
//! module).
//!

mod choice;
mod committee;
mod ledger;
mod manager;
mod payload;
mod status;
mod tally;

pub use self::{
    choice::{Choice, Options},
    committee::CommitteeId,
    ledger::{VotePlanLedger, VotePlanLedgerError},
    manager::{VoteError, VotePlanManager},
    payload::{Payload, PayloadType, TryFromIntError},
    status::{VotePlanStatus, VoteProposalStatus},
    tally::{Tally, TallyError, TallyResult, Weight},
};
