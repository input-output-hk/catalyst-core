use crate::db::{schema::voteplans, DB};
use crate::utils::datetime::unix_timestamp_to_datetime;
use chrono::{DateTime, Utc};
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
pub struct Voteplan {
    pub id: i32,
    pub chain_voteplan_id: String,
    #[serde(serialize_with = "crate::utils::serde::serialize_datetime_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_datetime_from_rfc3339")]
    pub chain_vote_start_time: DateTime<Utc>,
    #[serde(serialize_with = "crate::utils::serde::serialize_datetime_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_datetime_from_rfc3339")]
    pub chain_vote_end_time: DateTime<Utc>,
    #[serde(serialize_with = "crate::utils::serde::serialize_datetime_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_datetime_from_rfc3339")]
    pub chain_committee_end: DateTime<Utc>,
    pub chain_voteplan_payload: String,
    pub fund_id: i32,
}

impl Queryable<voteplans::SqlType, DB> for Voteplan {
    type Row = (
        // 0 -> id
        i32,
        // 1 > chain_voteplan_id
        String,
        // 2 -> chain_vote_start_time
        i64,
        // 3 -> chain_vote_end_time
        i64,
        // 4 -> chain_committee_end
        i64,
        // 5 -> chain_voteplan_payload
        String,
        // 6 -> fund_id
        i32,
    );

    fn build(row: Self::Row) -> Self {
        Self {
            id: row.0,
            chain_voteplan_id: row.1,
            chain_vote_start_time: unix_timestamp_to_datetime(row.2),
            chain_vote_end_time: unix_timestamp_to_datetime(row.3),
            chain_committee_end: unix_timestamp_to_datetime(row.4),
            chain_voteplan_payload: row.5,
            fund_id: row.6,
        }
    }
}

impl PartialEq for Voteplan {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.chain_voteplan_id == other.chain_voteplan_id
            && self.chain_vote_start_time.timestamp() == other.chain_vote_start_time.timestamp()
            && self.chain_vote_end_time.timestamp() == other.chain_vote_end_time.timestamp()
            && self.chain_committee_end.timestamp() == other.chain_committee_end.timestamp()
            && self.chain_voteplan_payload == other.chain_voteplan_payload
            && self.fund_id == other.fund_id
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use diesel::Insertable;

    impl Insertable<voteplans::SqlType> for &Voteplan {
        type Values = (
            // 0 > chain_voteplan_id
            String,
            // 1 -> chain_vote_start_time
            i64,
            // 2 -> chain_vote_end_time
            i64,
            // 3 -> chain_committee_end
            i64,
            // 4 -> chain_voteplan_payload
            String,
            // 5 -> fund_id
            i32,
        );

        fn values(self) -> Self::Values {
            (
                self.chain_voteplan_id.clone(),
                self.chain_vote_start_time.timestamp(),
                self.chain_vote_end_time.timestamp(),
                self.chain_committee_end.timestamp(),
                self.chain_voteplan_payload.clone(),
                self.fund_id,
            )
        }
    }
}
