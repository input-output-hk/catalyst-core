//! Defines the proposer details.
//!
use poem_openapi::{types::Example, Object};

/// Details about a proposer for a particular proposal.
#[derive(Object)]
#[oai(example = true)]
pub(crate) struct ProposerDetails {
    /// Name of the author/s of the proposal.
    name: String,

    /// Email contact address of the author's of the proposal.
    /// If not present, there is no known contact email for the authors.
    email: String,

    /// URL to a web resource with details about the author's of the proposal.
    url: String,

    /// The Payment Address the Funds requested will be paid to.
    /// Will not be included if the proposal does not request funds.
    #[oai(validator(max_length = 66, min_length = 66, pattern = "0x[0-9a-f]{64}"))]
    payment_key: String,
}

impl Example for ProposerDetails {
    fn example() -> Self {
        Self {
            name: "Name of the author/s of the proposal".to_string(),
            email: "Email contact address of the author/s of the proposal".to_string(),
            url: "URL to a web resource with details about the author/s of the proposal"
                .to_string(),
            payment_key: "0xb7a3c12dc0c8c748ab07525b701122b88bd78f600c76342d27f25e5f92444cde"
                .to_string(),
        }
    }
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
