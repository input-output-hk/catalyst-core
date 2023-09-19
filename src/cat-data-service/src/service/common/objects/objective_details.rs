//! Defines the objective details.
//!
use super::{reward_definition::RewardDefiniton, voter_group::VoterGroup};
use poem_openapi::{types::Example, Object};
use serde_json::Value;

#[derive(Object)]
#[oai(example = true)]
pub(crate) struct ObjectiveDetails {
    /// The valid voter groups for this voting event.
    groups: Vec<VoterGroup>,

    /// The Total Reward being offered for this Objective.
    /// Distribution of the Reward is determined under the rules of this Objective.
    /// If this field is not present there is no reward being offered for the Objective.
    #[oai(skip_serializing_if_is_none = true)]
    reward: Option<RewardDefiniton>,

    /// Objective Supplemental Data
    ///
    /// Extra Data which can be used to enrich the information shared about the Objective.
    /// All Information here is optional.
    /// If there is no supplemental information for the Objective this field is omitted.
    #[oai(skip_serializing_if_is_none = true)]
    supplemental: Option<Value>,
}

impl Example for ObjectiveDetails {
    fn example() -> Self {
        Self {
            groups: vec![VoterGroup::example()],
            reward: Some(RewardDefiniton::example()),
            supplemental: None,
        }
    }
}

impl TryFrom<event_db::types::objective::ObjectiveDetails> for ObjectiveDetails {
    type Error = String;
    fn try_from(value: event_db::types::objective::ObjectiveDetails) -> Result<Self, Self::Error> {
        let mut groups = Vec::new();
        for group in value.groups {
            groups.push(group.try_into()?);
        }
        let reward = if let Some(reward) = value.reward {
            Some(reward.try_into()?)
        } else {
            None
        };
        Ok(Self {
            groups,
            reward,
            supplemental: value.supplemental,
        })
    }
}
