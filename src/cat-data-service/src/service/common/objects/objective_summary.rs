use super::{objective_id::ObjectiveId, objective_type::ObjectiveType};
use poem_openapi::{types::Example, Object};

/// Summary off an Individual Objective.
#[derive(Object)]
pub(crate) struct ObjectiveSummary {
    /// The ID of this objective for the Voting Event.
    #[oai(validator(minimum(value = "0")))]
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

impl Example for ObjectiveSummary {
    fn example() -> Self {
        Self {
            id: ObjectiveId::example(),
            objective_type: ObjectiveType::example(),
            title: "Objective Title".to_string(),
            description: "Objective Description".to_string(),
            deleted: false,
        }
    }
}

impl TryFrom<event_db::types::objective::ObjectiveSummary> for ObjectiveSummary {
    type Error = String;
    fn try_from(value: event_db::types::objective::ObjectiveSummary) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            objective_type: value.objective_type.try_into()?,
            title: value.title,
            description: value.description,
            deleted: value.deleted,
        })
    }
}
