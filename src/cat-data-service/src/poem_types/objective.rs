use super::registration::VoterGroupId;
use poem_openapi::{NewType, Object};
use serde::Deserialize;
use serde_json::Value;

/// The Numeric ID of an Objective to be decided in a Voting Event.
#[derive(NewType, Deserialize)]
pub struct ObjectiveId(i32);

impl From<event_db::types::objective::ObjectiveId> for ObjectiveId {
    fn from(value: event_db::types::objective::ObjectiveId) -> Self {
        Self(value.0)
    }
}

#[derive(Object)]
pub struct ObjectiveType {
    /// Objective Type defines the rules of the objective.
    id: String,

    /// An explanaiton of the rules of this Objective Type.
    description: String,
}

impl From<event_db::types::objective::ObjectiveType> for ObjectiveType {
    fn from(value: event_db::types::objective::ObjectiveType) -> Self {
        Self {
            id: value.id,
            description: value.description,
        }
    }
}

/// Summary off an Individual Objective.
#[derive(Object)]
pub struct ObjectiveSummary {
    /// The ID of this objective for the Voting Event.
    id: ObjectiveId,

    /// The "Type" of Objective
    #[oai(rename = "type")]
    objective_type: ObjectiveType,

    /// The title for this Objective.
    title: String,

    /// Long form explanation of this particular objective.
    /// *May contain HTML Markup.*
    /// *May contain Links to external content or assets.*
    description: String,

    /// Whether this Objective has been deleted or not.
    deleted: bool,
}

impl From<event_db::types::objective::ObjectiveSummary> for ObjectiveSummary {
    fn from(value: event_db::types::objective::ObjectiveSummary) -> Self {
        Self {
            id: value.id.into(),
            objective_type: value.objective_type.into(),
            title: value.title,
            description: value.description,
            deleted: value.deleted,
        }
    }
}

#[derive(Object)]
pub struct RewardDefinition {
    /// Currency of the Reward.
    currency: String,

    /// The total value of the reward
    value: i64,
}

impl From<event_db::types::objective::RewardDefinition> for RewardDefinition {
    fn from(value: event_db::types::objective::RewardDefinition) -> Self {
        Self {
            currency: value.currency,
            value: value.value,
        }
    }
}

#[derive(Object)]
pub struct VoterGroup {
    #[oai(skip_serializing_if_is_none = true)]
    group: Option<VoterGroupId>,

    /// The identifier of voting power token used withing this group.
    /// All vote plans within a group are guaranteed to use the same token.
    #[oai(skip_serializing_if_is_none = true)]
    voting_token: Option<String>,
}

impl From<event_db::types::objective::VoterGroup> for VoterGroup {
    fn from(value: event_db::types::objective::VoterGroup) -> Self {
        Self {
            group: value.group.map(Into::into),
            voting_token: value.voting_token,
        }
    }
}

/// Individual Objective Details
#[derive(Object)]
pub struct ObjectiveDetails {
    /// The valid voter groups for this voting event.
    groups: Vec<VoterGroup>,

    /// The Total Reward being offered for this Objective.
    /// Distribution of the Reward is determined under the rules of this Objective.
    /// If this field is not present there is no reward being offered for the Objective.
    #[oai(skip_serializing_if_is_none = true)]
    reward: Option<RewardDefinition>,

    /// Objective Supplemental Data
    ///
    /// Extra Data which can be used to enrich the information shared about the Objective.
    /// All Information here is optional.
    /// If there is no supplemental information for the Objective this field is omitted.
    #[oai(skip_serializing_if_is_none = true)]
    supplemental: Option<Value>,
}

impl From<event_db::types::objective::ObjectiveDetails> for ObjectiveDetails {
    fn from(value: event_db::types::objective::ObjectiveDetails) -> Self {
        Self {
            groups: value.groups.into_iter().map(Into::into).collect(),
            reward: value.reward.map(Into::into),
            supplemental: value.supplemental,
        }
    }
}

#[derive(Object)]
pub struct Objective {
    #[oai(flatten)]
    summary: ObjectiveSummary,
    #[oai(flatten)]
    details: ObjectiveDetails,
}

impl From<event_db::types::objective::Objective> for Objective {
    fn from(value: event_db::types::objective::Objective) -> Self {
        Self {
            summary: value.summary.into(),
            details: value.details.into(),
        }
    }
}
