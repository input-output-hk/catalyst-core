use super::SerdeType;
use event_db::types::event::voting_status::VotingStatus;
use serde::ser::{Serialize, SerializeStruct, Serializer};

impl Serialize for SerdeType<VotingStatus> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("VotingStatus", 3)?;
        serializer.serialize_field("objective_id", &self.0.objective_id)?;
        serializer.serialize_field("open", &self.0.open)?;
        if let Some(settings) = &self.0.settings {
            serializer.serialize_field("settings", settings)?;
        }
        serializer.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use event_db::types::event::objective::ObjectiveId;
    use serde_json::json;

    #[test]
    fn voting_status_json_test() {
        let voting_status = SerdeType(VotingStatus {
            objective_id: ObjectiveId(1),
            open: false,
            settings: None,
        });

        let json = serde_json::to_value(&voting_status).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "objective_id": 1,
                    "open": false,
                }
            )
        );

        let voting_status = SerdeType(VotingStatus {
            objective_id: ObjectiveId(1),
            open: true,
            settings: Some("some settings".to_string()),
        });

        let json = serde_json::to_value(&voting_status).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "objective_id": 1,
                    "open": true,
                    "settings": "some settings",
                }
            )
        );
    }
}
