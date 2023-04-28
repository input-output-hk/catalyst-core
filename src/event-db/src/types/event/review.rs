use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Rating {
    pub review_type: i64,
    pub score: i64,
    pub note: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct AdvisorReview {
    pub assessor: String,
    pub ratings: Vec<Rating>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rating_json_test() {
        let rating = Rating {
            review_type: 1,
            score: 1,
            note: "note".to_string(),
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
