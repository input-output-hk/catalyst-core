//! Defines the full proposal information.
//!
use super::{proposal_details::ProposalDetails, proposal_summary::ProposalSummary};
use poem_openapi::{types::Example, Object};

/// Full Objective info.
#[derive(Object)]
#[oai(example = true)]
pub(crate) struct Proposal {
    #[oai(flatten)]
    summary: ProposalSummary,
    #[oai(flatten)]
    details: ProposalDetails,
}

impl Example for Proposal {
    fn example() -> Self {
        Self {
            summary: ProposalSummary::example(),
            details: ProposalDetails::example(),
        }
    }
}

impl From<event_db::types::proposal::Proposal> for Proposal {
    fn from(value: event_db::types::proposal::Proposal) -> Self {
        Self {
            summary: value.summary.into(),
            details: value.details.into(),
        }
    }
}
