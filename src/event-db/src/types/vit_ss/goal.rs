use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Goal {
    pub id: i32,
    pub goal_name: String,
    pub fund_id: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn goal_json_test() {
        let goal = Goal {
            id: 1,
            goal_name: "goal_name 1".to_string(),
            fund_id: 1,
        };

        let json = serde_json::to_value(&goal).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "goal_name": "goal_name 1",
                    "fund_id": 1
                }
            )
        )
    }
}
