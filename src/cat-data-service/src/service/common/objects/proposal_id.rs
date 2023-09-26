//! Defines the ID of a proposal.
//!
use poem_openapi::{types::Example, NewType};
use serde::Deserialize;

/// The Numeric ID of a Proposal.
#[derive(NewType, Deserialize)]
#[oai(example = true)]
pub(crate) struct ProposalId(i32);

impl Example for ProposalId {
    fn example() -> Self {
        Self(1)
    }
}

impl From<event_db::types::proposal::ProposalId> for ProposalId {
    fn from(value: event_db::types::proposal::ProposalId) -> Self {
        Self(value.0)
    }
}
