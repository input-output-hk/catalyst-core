use crate::types::utils::serialize_datetime_as_rfc3339;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Voteplan {
    pub id: i32,
    pub chain_voteplan_id: String,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub chain_vote_start_time: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub chain_vote_end_time: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub chain_committee_end_time: DateTime<Utc>,
    pub chain_voteplan_payload: String,
    pub chain_vote_encryption_key: String,
    pub fund_id: i32,
    pub token_identifier: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use serde_json::json;

    #[test]
    fn voteplan_json_test() {
        let voteplan = Voteplan {
            id: 1,
            chain_voteplan_id: "chain_voteplan_id 1".to_string(),
            chain_vote_start_time: DateTime::from_utc(NaiveDateTime::default(), Utc),
            chain_vote_end_time: DateTime::from_utc(NaiveDateTime::default(), Utc),
            chain_committee_end_time: DateTime::from_utc(NaiveDateTime::default(), Utc),
            chain_voteplan_payload: "chain_voteplan_payload 1".to_string(),
            chain_vote_encryption_key: "chain_vote_encryption_key 1".to_string(),
            fund_id: 1,
            token_identifier: "token_identifier 1".to_string(),
        };

        let json = serde_json::to_value(&voteplan).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "chain_voteplan_id": "chain_voteplan_id 1",
                    "chain_vote_start_time": "1970-01-01T00:00:00+00:00",
                    "chain_vote_end_time": "1970-01-01T00:00:00+00:00",
                    "chain_committee_end_time": "1970-01-01T00:00:00+00:00",
                    "chain_voteplan_payload": "chain_voteplan_payload 1",
                    "chain_vote_encryption_key": "chain_vote_encryption_key 1",
                    "fund_id": 1,
                    "token_identifier": "token_identifier 1",
                }
            )
        );
    }
}
