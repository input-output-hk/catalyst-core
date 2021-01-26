use crate::db::models::challenges::Challenge;

#[async_graphql::Object]
impl Challenge {
    /// Challenge ID
    pub async fn id(&self) -> i32 {
        self.id
    }

    /// Challenge title
    pub async fn title(&self) -> &str {
        &self.title
    }

    /// Challenge description
    pub async fn description(&self) -> &str {
        &self.description
    }

    /// Challenge rewards
    pub async fn rewards_total(&self) -> i64 {
        self.rewards_total
    }

    /// Challenge related fund id
    pub async fn fund_id(&self) -> i32 {
        self.fund_id
    }

    /// Challenge information link
    pub async fn challenge_url(&self) -> &str {
        &self.challenge_url
    }
}
