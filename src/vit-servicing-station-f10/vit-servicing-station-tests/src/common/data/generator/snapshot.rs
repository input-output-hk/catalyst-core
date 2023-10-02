use std::collections::HashMap;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::goals::Goal;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_lib::db::models::{
    api_tokens::ApiTokenData, challenges::Challenge, funds::Fund, voteplans::Voteplan,
};

#[derive(Debug, Clone)]
pub struct Snapshot {
    funds: Vec<Fund>,
    proposals: Vec<FullProposalInfo>,
    challenges: Vec<Challenge>,
    tokens: HashMap<String, ApiTokenData>,
    voteplans: Vec<Voteplan>,
    reviews: Vec<AdvisorReview>,
    goals: Vec<Goal>,
}

impl Snapshot {
    pub fn new(
        funds: Vec<Fund>,
        proposals: Vec<FullProposalInfo>,
        challenges: Vec<Challenge>,
        tokens: HashMap<String, ApiTokenData>,
        voteplans: Vec<Voteplan>,
        reviews: Vec<AdvisorReview>,
        goals: Vec<Goal>,
    ) -> Self {
        Self {
            funds,
            proposals,
            challenges,
            tokens,
            voteplans,
            reviews,
            goals,
        }
    }

    pub fn funds(&self) -> Vec<Fund> {
        self.funds.clone()
    }

    pub fn proposals(&self) -> Vec<FullProposalInfo> {
        self.proposals.clone()
    }

    pub fn tokens(&self) -> HashMap<String, ApiTokenData> {
        self.tokens.clone()
    }

    pub fn voteplans(&self) -> Vec<Voteplan> {
        self.voteplans.clone()
    }

    pub fn funds_mut(&mut self) -> &mut Vec<Fund> {
        &mut self.funds
    }

    pub fn proposals_mut(&mut self) -> &mut Vec<FullProposalInfo> {
        &mut self.proposals
    }

    pub fn voteplans_mut(&mut self) -> &mut Vec<Voteplan> {
        &mut self.voteplans
    }

    pub fn proposal_by_id(&self, id: &str) -> Option<&FullProposalInfo> {
        self.proposals
            .iter()
            .find(|x| x.proposal.proposal_id.eq(id))
    }

    pub fn fund_by_id(&self, id: i32) -> Option<&Fund> {
        self.funds.iter().find(|x| x.id == id)
    }

    pub fn any_token(&self) -> (String, ApiTokenData) {
        let (hash, token) = self.tokens.iter().next().unwrap();
        (hash.to_string(), token.clone())
    }

    pub fn token_hash(&self) -> String {
        self.any_token().0
    }

    pub fn challenges(&self) -> Vec<Challenge> {
        self.challenges.clone()
    }

    pub fn challenges_mut(&mut self) -> &mut Vec<Challenge> {
        &mut self.challenges
    }

    pub fn advisor_reviews(&self) -> Vec<AdvisorReview> {
        self.reviews.clone()
    }

    pub fn advisor_reviews_mut(&mut self) -> &mut Vec<AdvisorReview> {
        &mut self.reviews
    }

    pub fn goals(&self) -> Vec<Goal> {
        self.goals.clone()
    }
}
