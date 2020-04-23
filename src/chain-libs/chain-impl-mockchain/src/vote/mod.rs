//! cover all that relate to the voting part of things
//! (except for the certificate that are in the certificate
//! module).
//!

mod committee;
mod ledger;
mod manager;

pub use self::{
    committee::CommitteeId,
    ledger::{VotePlanLedger, VotePlanLedgerError},
    manager::{VoteError, VotePlanManager},
};
