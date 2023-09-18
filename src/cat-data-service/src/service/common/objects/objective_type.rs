use super::objective_types::ObjectiveTypes;
use poem_openapi::{types::Example, Object};

/// Objective type definition.
#[derive(Object)]
pub(crate) struct ObjectiveType {
    id: ObjectiveTypes,

    /// An explanation of the rules of this Objective Type.
    description: String,
}

impl Example for ObjectiveType {
    fn example() -> Self {
        Self {
            id: ObjectiveTypes::Simple,
            description: "Objective type description".to_string(),
        }
    }
}

impl TryFrom<event_db::types::objective::ObjectiveType> for ObjectiveType {
    type Error = String;
    fn try_from(value: event_db::types::objective::ObjectiveType) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.try_into()?,
            description: value.description,
        })
    }
}
