//! Defines the proposal details.
//!
use super::proposer_details::ProposerDetails;
use poem_openapi::{types::Example, Object};
use serde_json::Value;

/// Details of a particular Proposal.
#[derive(Object)]
pub(crate) struct ProposalDetails {
    /// The amount of funds requested by this proposal.
    /// In the denomination of the Objectives Reward.
    /// If not present, then this proposal is not requesting any funds.
    #[oai(validator(minimum(value = "0")))]
    funds: i64,

    /// URL to a web page with details on this proposal.
    url: String,

    /// Link to extra files associated with this proposal.
    /// Only included if there are linked files.
    files: String,

    /// List of all proposers making this proposal.
    proposer: Vec<ProposerDetails>,

    /// Proposal Supplemental Data
    ///
    /// Extra Data which can be used to enrich the information shared about the Proposal.
    /// All Information here is optional.
    /// If there is no supplemental information for the Proposal this field is omitted.
    #[oai(skip_serializing_if_is_none = true)]
    supplemental: Option<Value>,
}

impl Example for ProposalDetails {
    fn example() -> Self {
        Self {
            funds: 0,
            url: "URL to a web page with details on this proposal".to_string(),
            files: "Link to extra files associated with this proposal".to_string(),
            proposer: vec![ProposerDetails::example()],
            supplemental: None,
        }
    }
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
