//! cover all that relate to the voting part of things
//! (except for the certificate that are in the certificate
//! module).
//!

mod committee;
mod manager;

pub use self::{committee::CommitteeId, manager::VotePlanManager};
