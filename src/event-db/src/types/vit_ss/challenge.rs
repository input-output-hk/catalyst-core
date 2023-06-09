use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ChallengeHighlights {
    pub sponsor: String,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Challenge {
    // this is used only to retain the original insert order
    pub internal_id: i32,
    pub id: i32,
    pub challenge_type: String,
    pub title: String,
    pub description: String,
    pub rewards_total: i64,
    pub proposers_rewards: i64,
    pub fund_id: i32,
    pub challenge_url: String,
    pub highlights: Option<ChallengeHighlights>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn challenge_json_test() {
        let challenge = Challenge {
            internal_id: 1,
            id: 1,
            challenge_type: "catalyst-simple".to_string(),
            title: "title 1".to_string(),
            description: "description 1".to_string(),
            rewards_total: 1,
            proposers_rewards: 1,
            fund_id: 1,
            challenge_url: "url 1".to_string(),
            highlights: Some(ChallengeHighlights {
                sponsor: "sponsor 1".to_string(),
            }),
        };

        let json = serde_json::to_value(&challenge).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "internal_id": 1,
                    "id": 1,
                    "challenge_type": "catalyst-simple",
                    "title": "title 1",
                    "description": "description 1",
                    "rewards_total": 1,
                    "proposers_rewards": 1,
                    "fund_id": 1,
                    "challenge_url": "url 1",
                    "highlights": {
                        "sponsor": "sponsor 1",
                    },
                }
            )
        );
    }
}
