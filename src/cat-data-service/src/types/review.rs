use super::SerdeType;
use event_db::types::event::review::{AdvisorReview, Rating, ReviewType};
use serde::{
    ser::{SerializeStruct, Serializer},
    Serialize,
};

impl Serialize for SerdeType<&ReviewType> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("ReviewType", 8)?;
        serializer.serialize_field("id", &self.id)?;
        serializer.serialize_field("name", &self.name)?;
        if let Some(description) = &self.description {
            serializer.serialize_field("description", description)?;
        }
        serializer.serialize_field("min", &self.min)?;
        serializer.serialize_field("max", &self.max)?;
        serializer.serialize_field("map", &self.map)?;
        if let Some(note) = &self.note {
            serializer.serialize_field("note", note)?;
        }
        if let Some(group) = &self.group {
            serializer.serialize_field("group", group)?;
        }
        serializer.end()
    }
}

impl Serialize for SerdeType<ReviewType> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&Rating> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("Rating", 3)?;
        serializer.serialize_field("review_type", &self.review_type)?;
        serializer.serialize_field("score", &self.score)?;
        if let Some(note) = &self.note {
            serializer.serialize_field("note", note)?;
        }
        serializer.end()
    }
}

impl Serialize for SerdeType<Rating> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&AdvisorReview> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("AdvisorReview", 2)?;
        serializer.serialize_field("assessor", &self.assessor)?;
        serializer.serialize_field(
            "ratings",
            &self.ratings.iter().map(SerdeType).collect::<Vec<_>>(),
        )?;
        serializer.end()
    }
}

impl Serialize for SerdeType<AdvisorReview> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn review_type_json_test() {
        let review_type = SerdeType(ReviewType {
            id: 1,
            name: "name".to_string(),
            description: Some("description".to_string()),
            min: 1,
            max: 2,
            note: Some(true),
            map: vec![],
            group: Some("group".to_string()),
        });

        let json = serde_json::to_value(&review_type).unwrap();
        assert_eq!(
            json,
            json!(
                    {
                        "id": 1,
                        "name": "name",
                        "description": "description",
                        "min": 1,
                        "max": 2,
                        "note": true,
                        "map": [],
                        "group": "group",
                    }
            )
        );

        let review_type = SerdeType(ReviewType {
            id: 1,
            name: "name".to_string(),
            description: None,
            min: 1,
            max: 2,
            note: None,
            map: vec![],
            group: None,
        });

        let json = serde_json::to_value(&review_type).unwrap();
        assert_eq!(
            json,
            json!(
                    {
                        "id": 1,
                        "name": "name",
                        "min": 1,
                        "max": 2,
                        "map": [],
                    }
            )
        );
    }

    #[test]
    fn rating_json_test() {
        let rating = SerdeType(Rating {
            review_type: 1,
            score: 1,
            note: Some("note".to_string()),
        });

        let json = serde_json::to_value(&rating).unwrap();
        assert_eq!(
            json,
            json!(
                    {
                        "review_type": 1,
                        "score": 1,
                        "note": "note",
                    }
            )
        );

        let rating = SerdeType(Rating {
            review_type: 1,
            score: 1,
            note: None,
        });

        let json = serde_json::to_value(&rating).unwrap();
        assert_eq!(
            json,
            json!(
                    {
                        "review_type": 1,
                        "score": 1,
                    }
            )
        );
    }

    #[test]
    fn advisor_review_json_test() {
        let advisor_review = SerdeType(AdvisorReview {
            assessor: "alice".to_string(),
            ratings: vec![],
        });

        let json = serde_json::to_value(&advisor_review).unwrap();
        assert_eq!(
            json,
            json!(
                    {
                        "assessor": "alice",
                        "ratings": [],
                    }
            )
        )
    }
}
