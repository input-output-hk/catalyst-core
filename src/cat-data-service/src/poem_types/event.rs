use poem_openapi::NewType;
use serde::Deserialize;

#[derive(NewType, Deserialize)]
pub struct EventId(pub i32);

impl Into<event_db::types::event::EventId> for EventId {
    fn into(self) -> event_db::types::event::EventId {
        event_db::types::event::EventId(self.0)
    }
}
