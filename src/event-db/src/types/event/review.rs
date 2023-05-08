use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewType {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub min: i32,
    pub max: i32,
    pub map: Vec<String>,
    pub note: Option<bool>,
    pub group: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rating {
    pub review_type: i32,
    pub score: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AdvisorReview {
    pub assessor: String,
    pub ratings: Vec<Rating>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn review_type_json_test() {
        let review_type = ReviewType {
            id: 1,
            name: "name".to_string(),
            description: Some("description".to_string()),
            min: 1,
            max: 2,
            note: Some(true),
            map: vec!["map".to_string()],
            group: Some("group".to_string()),
        };
        let json = serde_json::to_value(&review_type).unwrap();

        assert_eq!(
            json,
            json!({
                "id": 1,
                "name": "name",
                "description": "description",
                "min": 1,
                "max": 2,
                "note": true,
                "map": ["map"],
                "group": "group",
            })
        )
    }

    #[test]
    fn rating_json_test() {
        let rating = Rating {
            review_type: 1,
            score: 1,
            note: Some("note".to_string()),
        };

        let json = serde_json::to_value(&rating).unwrap();
        assert_eq!(
            json,
            json!({
                "review_type": 1,
                "score": 1,
                "note": "note",
            }),
        )
    }

    #[test]
    fn advisor_review_json_test() {
        let advisor_review = AdvisorReview {
            assessor: "alice".to_string(),
            ratings: vec![],
        };

        let json = serde_json::to_value(&advisor_review).unwrap();
        assert_eq!(
            json,
            json!({
                "assessor": "alice",
                "ratings": [],
            })
        )
    }
}
