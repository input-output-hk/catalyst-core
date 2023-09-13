use poem_openapi::{NewType, Object};
use serde::Deserialize;
use serde_json::Value;

#[derive(NewType, Deserialize)]
pub struct ProposalId(i32);

impl From<event_db::types::proposal::ProposalId> for ProposalId {
    fn from(value: event_db::types::proposal::ProposalId) -> Self {
        Self(value.0)
    }
}

/// Summary of a Proposal for an Objective.
#[derive(Object)]
pub struct ProposalSummary {
    /// The ID of this proposal.
    id: ProposalId,

    /// Short title of the proposal.
    title: String,

    /// Brief description of the proposal.
    summary: String,

    /// Whether this Proposal has been deleted or not.
    deleted: bool,
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

/// Details about a proposer for a particular proposal.
#[derive(Object)]
pub struct ProposerDetails {
    /// Name of the author/s of the proposal.
    name: String,

    /// Email contact address of the author's of the proposal.
    /// If not present, there is no known contact email for the authors.
    email: String,

    /// URL to a web resource with details about the author's of the proposal.
    url: String,

    /// The Payment Address the Funds requested will be paid to.
    /// Will not be included if the proposal does not request funds.
    payment_key: String,
}

impl From<event_db::types::proposal::ProposerDetails> for ProposerDetails {
    fn from(value: event_db::types::proposal::ProposerDetails) -> Self {
        Self {
            name: value.name,
            email: value.email,
            url: value.url,
            payment_key: value.payment_key,
        }
    }
}

/// Details of a particular Proposal.
#[derive(Object)]
pub struct ProposalDetails {
    /// The amount of funds requested by this proposal.
    /// In the denomination of the Objectives Reward.
    /// If not present, then this proposal is not requesting any funds.
    funds: i64,

    /// URL to a web page with details on this proposal.
    url: String,

    /// Link to extra files associated with this proposal.
    /// Only included if there are linked files.
    files: String,

    /// List of all proposers making this proposal.
    proposer: Vec<ProposerDetails>,

    /// Additional Information related to the proposal.
    #[oai(skip_serializing_if_is_none = true)]
    supplemental: Option<Value>,
}

impl From<event_db::types::proposal::ProposalDetails> for ProposalDetails {
    fn from(value: event_db::types::proposal::ProposalDetails) -> Self {
        Self {
            funds: value.funds,
            url: value.url,
            files: value.files,
            proposer: value.proposer.into_iter().map(Into::into).collect(),
            supplemental: value.supplemental,
        }
    }
}

#[derive(Object)]
pub struct Proposal {
    #[oai(flatten)]
    summary: ProposalSummary,
    #[oai(flatten)]
    details: ProposalDetails,
}

impl From<event_db::types::proposal::Proposal> for Proposal {
    fn from(value: event_db::types::proposal::Proposal) -> Self {
        Self {
            summary: value.summary.into(),
            details: value.details.into(),
        }
    }
}
