use poem_openapi::{types::Example, NewType};
use serde::Deserialize;

/// The Numeric ID of a Voting Event
#[derive(NewType, Deserialize)]
#[oai(example = true)]
pub struct EventId(pub i32);

impl Example for EventId {
    fn example() -> Self {
        Self(11)
    }
}

impl From<EventId> for event_db::types::event::EventId {
    fn from(event_id: EventId) -> Self {
        event_db::types::event::EventId(event_id.0)
    }
}
