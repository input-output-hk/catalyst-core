//! Defines the voter group details.
//!
use super::voter_group_id::VoterGroupId;
use poem_openapi::{types::Example, Object};

/// Voter group details.
#[derive(Object)]
#[oai(example = true)]
pub(crate) struct VoterGroup {
    #[oai(skip_serializing_if_is_none = true)]
    group: Option<VoterGroupId>,

    /// The identifier of voting power token used withing this group.
    /// All vote plans within a group are guaranteed to use the same token.
    #[oai(skip_serializing_if_is_none = true)]
    voting_token: Option<String>,
}

impl Example for VoterGroup {
    fn example() -> Self {
        Self {
            group: Some(VoterGroupId::Direct),
            voting_token: Some("voting_token id".to_string()),
        }
    }
}

impl TryFrom<event_db::types::objective::VoterGroup> for VoterGroup {
    type Error = String;
    fn try_from(value: event_db::types::objective::VoterGroup) -> Result<Self, Self::Error> {
        let group = if let Some(group) = value.group {
            Some(group.try_into()?)
        } else {
            None
        };
        Ok(Self {
            group,
            voting_token: value.voting_token,
        })
    }
}
