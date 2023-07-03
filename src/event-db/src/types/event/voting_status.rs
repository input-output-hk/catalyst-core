use super::objective::ObjectiveId;
use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct VotingStatus {
    pub objective_id: ObjectiveId,
    pub open: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn voting_status_json_test() {
        let voting_status = VotingStatus {
            objective_id: ObjectiveId(1),
            open: false,
            settings: None,
        };

        let json = serde_json::to_value(&voting_status).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "objective_id": 1,
                    "open": false,
                }
            )
        )
    }
}
