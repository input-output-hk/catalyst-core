//! Defines the proposal summary.
//!
use super::proposal_id::ProposalId;
use poem_openapi::{types::Example, Object};

/// Summary of a Proposal.
#[derive(Object)]
#[oai(example = true)]
pub(crate) struct ProposalSummary {
    /// The ID of this proposal.
    id: ProposalId,

    /// Short title of the proposal.
    title: String,

    /// Brief description of the proposal.
    summary: String,

    /// Whether this Proposal has been deleted or not.
    deleted: bool,
}

impl Example for ProposalSummary {
    fn example() -> Self {
        Self {
            id: ProposalId::example(),
            title: "Proposal Title".to_string(),
            summary: "Proposal Summary".to_string(),
            deleted: false,
        }
    }
}

impl From<event_db::types::proposal::ProposalSummary> for ProposalSummary {
    fn from(value: event_db::types::proposal::ProposalSummary) -> Self {
        Self {
            id: value.id.into(),
            title: value.title,
            summary: value.summary,
            deleted: value.deleted,
        }
    }
}
