use crate::db::models::proposals::{Category, FullProposalInfo, Proposer};
use crate::db::models::vote_options::VoteOptions;
use crate::utils::datetime::unix_timestamp_to_datetime;
use async_graphql::OutputJson;

#[async_graphql::Object]
impl Category {
    pub async fn category_id(&self) -> &str {
        &self.category_id
    }

    pub async fn category_name(&self) -> &str {
        &self.category_name
    }

    pub async fn category_description(&self) -> &str {
        &self.category_description
    }
}

#[async_graphql::Object]
impl Proposer {
    pub async fn proposer_name(&self) -> &str {
        &self.proposer_name
    }

    pub async fn proposer_email(&self) -> &str {
        &self.proposer_email
    }

    pub async fn proposer_url(&self) -> &str {
        &self.proposer_url
    }

    pub async fn proposer_relevant_experience(&self) -> &str {
        &self.proposer_relevant_experience
    }
}

#[async_graphql::Object]
impl FullProposalInfo {
    pub async fn internal_id(&self) -> i32 {
        self.proposal.internal_id
    }

    pub async fn category(&self) -> &Category {
        &self.proposal.proposal_category
    }

    pub async fn proposal_id(&self) -> &str {
        &self.proposal.proposal_id
    }

    pub async fn proposal_title(&self) -> &str {
        &self.proposal.proposal_title
    }

    pub async fn proposal_summary(&self) -> &str {
        &self.proposal.proposal_summary
    }

    pub async fn proposal_public_key(&self) -> &str {
        &self.proposal.proposal_public_key
    }

    pub async fn proposal_funds(&self) -> i64 {
        self.proposal.proposal_funds
    }

    pub async fn proposal_url(&self) -> &str {
        &self.proposal.proposal_url
    }

    pub async fn proposal_files_url(&self) -> &str {
        &self.proposal.proposal_files_url
    }

    pub async fn proposal_impact_score(&self) -> i64 {
        self.proposal.proposal_impact_score
    }

    pub async fn proposer(&self) -> &Proposer {
        &self.proposal.proposer
    }

    pub async fn chain_proposal_id(&self) -> String {
        String::from_utf8(self.proposal.chain_proposal_id.clone()).unwrap()
    }

    pub async fn chain_voteplan_id(&self) -> &str {
        &self.proposal.chain_voteplan_id
    }

    pub async fn chain_proposal_index(&self) -> i64 {
        self.proposal.chain_proposal_index
    }

    pub async fn chain_voteplan_payload(&self) -> &str {
        &self.proposal.chain_voteplan_payload
    }

    pub async fn chain_vote_encryption_key(&self) -> &str {
        &self.proposal.chain_vote_encryption_key
    }

    pub async fn chain_vote_start_time(&self) -> String {
        unix_timestamp_to_datetime(self.proposal.chain_vote_start_time).to_rfc3339()
    }

    pub async fn chain_vote_end_time(&self) -> String {
        unix_timestamp_to_datetime(self.proposal.chain_vote_end_time).to_rfc3339()
    }

    pub async fn chain_committee_end_time(&self) -> String {
        unix_timestamp_to_datetime(self.proposal.chain_committee_end_time).to_rfc3339()
    }

    pub async fn chain_vote_options(&self) -> OutputJson<&VoteOptions> {
        OutputJson(&self.proposal.chain_vote_options)
    }

    pub async fn fund_id(&self) -> i32 {
        self.proposal.fund_id
    }

    pub async fn challenge_id(&self) -> i32 {
        self.proposal.challenge_id
    }

    pub async fn challenge_type(&self) -> String {
        self.challenge_type.to_string()
    }
}
