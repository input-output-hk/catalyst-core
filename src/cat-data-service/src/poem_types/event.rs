use poem_openapi::NewType;
use serde::Deserialize;

#[derive(NewType, Deserialize)]
pub struct EventId(pub i32);

impl From<EventId> for event_db::types::event::EventId {
    fn from(event_id: EventId) -> Self {
        event_db::types::event::EventId(event_id.0)
    }
}
