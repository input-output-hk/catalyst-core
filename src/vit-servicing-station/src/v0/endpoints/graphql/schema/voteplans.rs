use crate::db::models::voteplans::Voteplan;

#[async_graphql::Object]
impl Voteplan {
    pub async fn id(&self) -> i32 {
        self.id
    }

    pub async fn chain_voteplan_id(&self) -> &str {
        &self.chain_voteplan_id
    }

    pub async fn chain_vote_start_time(&self) -> &str {
        &self.chain_vote_start_time
    }

    pub async fn chain_vote_end_time(&self) -> &str {
        &self.chain_vote_end_time
    }

    pub async fn chain_committee_end(&self) -> &str {
        &self.chain_committee_end
    }

    pub async fn chain_voteplan_payload(&self) -> &str {
        &self.chain_voteplan_payload
    }

    pub async fn fund_id(&self) -> i32 {
        self.fund_id
    }
}
