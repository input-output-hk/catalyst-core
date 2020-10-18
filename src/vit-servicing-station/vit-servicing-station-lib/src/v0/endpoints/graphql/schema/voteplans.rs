use crate::db::models::voteplans::Voteplan;
use crate::utils::datetime::unix_timestamp_to_datetime;

#[async_graphql::Object]
impl Voteplan {
    pub async fn id(&self) -> i32 {
        self.id
    }

    pub async fn chain_voteplan_id(&self) -> &str {
        &self.chain_voteplan_id
    }

    pub async fn chain_vote_start_time(&self) -> String {
        unix_timestamp_to_datetime(self.chain_vote_start_time).to_rfc3339()
    }

    pub async fn chain_vote_end_time(&self) -> String {
        unix_timestamp_to_datetime(self.chain_vote_end_time).to_rfc3339()
    }

    pub async fn chain_committee_end_time(&self) -> String {
        unix_timestamp_to_datetime(self.chain_committee_end_time).to_rfc3339()
    }

    pub async fn chain_voteplan_payload(&self) -> &str {
        &self.chain_voteplan_payload
    }

    pub async fn chain_vote_encryption_key(&self) -> &str {
        &self.chain_vote_encryption_key
    }

    pub async fn fund_id(&self) -> i32 {
        self.fund_id
    }
}
