use crate::db::schema::voteplans;
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Queryable)]
pub struct Voteplan {
    pub id: i32,
    #[serde(alias = "chainVoteplanId")]
    pub chain_voteplan_id: String,
    #[serde(alias = "chainVoteStartTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_start_time: i64,
    #[serde(alias = "chainVoteEndTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_vote_end_time: i64,
    #[serde(alias = "chainCommitteeEndTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub chain_committee_end_time: i64,
    #[serde(alias = "chainVoteplanPayload")]
    pub chain_voteplan_payload: String,
    #[serde(alias = "chainVoteEncryptionKey")]
    pub chain_vote_encryption_key: String,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
}

// This warning is disabled here. Values is only referenced as a type here. It should be ok not to
// split the types definitions.
#[allow(clippy::type_complexity)]
impl Insertable<voteplans::table> for Voteplan {
    type Values = (
        diesel::dsl::Eq<voteplans::chain_voteplan_id, String>,
        diesel::dsl::Eq<voteplans::chain_vote_start_time, i64>,
        diesel::dsl::Eq<voteplans::chain_vote_end_time, i64>,
        diesel::dsl::Eq<voteplans::chain_committee_end_time, i64>,
        diesel::dsl::Eq<voteplans::chain_voteplan_payload, String>,
        diesel::dsl::Eq<voteplans::chain_vote_encryption_key, String>,
        diesel::dsl::Eq<voteplans::fund_id, i32>,
    );

    fn values(self) -> Self::Values {
        (
            voteplans::chain_voteplan_id.eq(self.chain_voteplan_id),
            voteplans::chain_vote_start_time.eq(self.chain_vote_start_time),
            voteplans::chain_vote_end_time.eq(self.chain_vote_end_time),
            voteplans::chain_committee_end_time.eq(self.chain_committee_end_time),
            voteplans::chain_voteplan_payload.eq(self.chain_voteplan_payload),
            voteplans::chain_vote_encryption_key.eq(self.chain_vote_encryption_key),
            voteplans::fund_id.eq(self.fund_id),
        )
    }
}

#[cfg(test)]
pub mod test {
    use crate::db::{models::voteplans::Voteplan, schema::voteplans, DbConnectionPool};
    use diesel::{ExpressionMethods, RunQueryDsl};
    use time::OffsetDateTime;

    pub fn get_test_voteplan_with_fund_id(fund_id: i32) -> Voteplan {
        Voteplan {
            id: 1,
            chain_voteplan_id: format!("test_vote_plan{fund_id}"),
            chain_vote_start_time: OffsetDateTime::now_utc().unix_timestamp(),
            chain_vote_end_time: OffsetDateTime::now_utc().unix_timestamp(),
            chain_committee_end_time: OffsetDateTime::now_utc().unix_timestamp(),
            chain_voteplan_payload: "foopayload".to_string(),
            chain_vote_encryption_key: "enckey".to_string(),
            fund_id,
        }
    }

    pub fn populate_db_with_voteplan(voteplan: &Voteplan, pool: &DbConnectionPool) {
        let connection = pool.get().unwrap();
        let values = (
            voteplans::chain_voteplan_id.eq(voteplan.chain_voteplan_id.clone()),
            voteplans::chain_vote_start_time.eq(voteplan.chain_vote_start_time),
            voteplans::chain_vote_end_time.eq(voteplan.chain_vote_end_time),
            voteplans::chain_committee_end_time.eq(voteplan.chain_committee_end_time),
            voteplans::chain_voteplan_payload.eq(voteplan.chain_voteplan_payload.clone()),
            voteplans::chain_vote_encryption_key.eq(voteplan.chain_vote_encryption_key.clone()),
            voteplans::fund_id.eq(voteplan.fund_id),
        );
        diesel::insert_into(voteplans::table)
            .values(values)
            .execute(&connection)
            .unwrap();
    }
}
