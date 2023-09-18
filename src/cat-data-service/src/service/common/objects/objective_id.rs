use poem_openapi::{types::Example, NewType};
use serde::Deserialize;

/// The Numeric ID of an Objective to be decided in a Voting Event.
#[derive(NewType, Deserialize)]
pub(crate) struct ObjectiveId(i32);

impl Example for ObjectiveId {
    fn example() -> Self {
        Self(1)
    }
}

impl From<event_db::types::objective::ObjectiveId> for ObjectiveId {
    fn from(value: event_db::types::objective::ObjectiveId) -> Self {
        Self(value.0)
    }
}
