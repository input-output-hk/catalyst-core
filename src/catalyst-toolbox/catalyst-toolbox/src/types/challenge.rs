use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub id: i32,
    pub title: String,
    #[serde(alias = "rewardsTotal")]
    pub rewards_total: i64,
    #[serde(alias = "proposersRewards")]
    pub proposers_rewards: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_json_test() {
        let json = serde_json::json!({
            "id": 1,
            "title": "test",
            "rewards_total": 1,
            "proposers_rewards": 1,
            "not_challenge_data": "some"
        });

        let challenge: Challenge = serde_json::from_value(json).unwrap();

        assert_eq!(
            challenge,
            Challenge {
                id: 1,
                title: "test".to_string(),
                rewards_total: 1,
                proposers_rewards: 1,
            }
        );
    }
}