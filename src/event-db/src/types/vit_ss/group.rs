use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Group {
    pub fund_id: i32,
    pub token_identifier: String,
    pub group_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn group_json_test() {
        let group = Group {
            fund_id: 1,
            token_identifier: "token_identifier 1".to_string(),
            group_id: "group_id 1".to_string(),
        };

        let json = serde_json::to_value(&group).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "fund_id": 1,
                    "token_identifier": "token_identifier 1",
                    "group_id": "group_id 1"
                }
            )
        )
    }
}
