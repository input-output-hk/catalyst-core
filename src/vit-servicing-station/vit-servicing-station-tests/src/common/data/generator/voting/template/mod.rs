mod arbitrary;
mod external;

pub use arbitrary::ArbitraryValidVotingTemplateGenerator;
pub use external::{
    parse_challenges, parse_funds, parse_proposals, parse_reviews,
    ExternalValidVotingTemplateGenerator, TemplateLoad,
};
use serde::{Deserialize, Serialize};
use vit_servicing_station_lib::db::models::challenges::ChallengeHighlights;
use vit_servicing_station_lib::db::models::community_advisors_reviews::ReviewRanking;
use vit_servicing_station_lib::db::models::proposals::{ChallengeType, ProposalChallengeInfo};
use vit_servicing_station_lib::db::models::vote_options::VoteOptions;

#[derive(Serialize, Deserialize, Clone)]
pub struct FundTemplate {
    pub id: i32,
    pub goal: String,
    pub rewards_info: String,
    pub threshold: Option<u32>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ProposalTemplate {
    pub internal_id: String,
    pub category_name: String,
    pub proposal_id: String,
    pub proposal_title: String,
    #[serde(default)]
    pub proposal_summary: String,
    pub proposal_funds: String,
    pub proposal_url: String,
    pub proposal_impact_score: String,
    #[serde(default)]
    pub files_url: String,
    pub proposer_name: String,
    #[serde(default)]
    pub proposer_url: String,
    #[serde(default)]
    pub proposer_relevant_experience: String,
    #[serde(
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_vote_options_from_string"
    )]
    pub chain_vote_options: VoteOptions,
    pub chain_vote_type: String,
    pub challenge_id: Option<String>,
    pub challenge_type: ChallengeType,
    #[serde(flatten)]
    pub proposal_challenge_info: ProposalChallengeInfo,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChallengeTemplate {
    pub internal_id: i32,
    pub id: String,
    pub challenge_type: ChallengeType,
    pub title: String,
    pub description: String,
    pub rewards_total: String,
    pub proposers_rewards: String,
    pub challenge_url: String,
    pub fund_id: Option<String>,
    pub highlight: Option<ChallengeHighlights>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReviewTemplate {
    pub id: Option<String>,
    pub proposal_id: String,
    pub assessor: String,
    pub impact_alignment_rating_given: i32,
    pub impact_alignment_note: String,
    pub feasibility_rating_given: i32,
    pub feasibility_note: String,
    pub auditability_rating_given: i32,
    pub auditability_note: String,
    pub ranking: ReviewRanking,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProposalChallengeInfoTemplate {
    pub id: i32,
}

pub trait ValidVotingTemplateGenerator {
    fn next_proposal(&mut self) -> ProposalTemplate;
    fn next_challenge(&mut self) -> ChallengeTemplate;
    fn next_fund(&mut self) -> FundTemplate;
    fn next_review(&mut self) -> ReviewTemplate;
}
