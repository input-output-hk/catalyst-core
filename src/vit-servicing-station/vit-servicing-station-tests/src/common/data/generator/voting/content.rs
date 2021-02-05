use crate::common::data::ArbitraryGenerator;
use fake::{
    faker::company::en::{Buzzword, CatchPhase},
    faker::lorem::en::*,
    Fake,
};
use vit_servicing_station_lib::db::models::proposals::{Category, Proposer};
pub struct FundContent {
    pub goal: String,
    pub rewards_info: String,
}

pub struct ProposalContent {
    pub category: Category,
    pub title: String,
    pub summary: String,
    pub problem: String,
    pub solution: String,
    pub funds: i64,
    pub url: String,
    pub impact_score: i64,
    pub files_url: String,
    pub proposer: Proposer,
}

pub struct ChallengeContent {
    pub title: String,
    pub description: String,
    pub rewards_total: u32,
    pub challenge_url: String,
}

pub trait ValidVotingDataContent {
    fn next_proposal(&mut self) -> ProposalContent;
    fn next_challenge(&mut self) -> ChallengeContent;
    fn next_fund(&mut self) -> FundContent;
}

impl ValidVotingDataContent for ArbitraryValidVotingDataContent {
    fn next_proposal(&mut self) -> ProposalContent {
        let proposal_url = self.generator.gen_http_address();
        ProposalContent {
            category: self.generator.proposal_category(),
            title: CatchPhase().fake::<String>(),
            summary: CatchPhase().fake::<String>(),
            problem: Buzzword().fake::<String>(),
            solution: CatchPhase().fake::<String>(),
            funds: self.generator.proposal_fund(),
            url: proposal_url.to_string(),
            impact_score: self.generator.impact_score(),
            files_url: format!("{}/files", proposal_url),
            proposer: self.generator.proposer(),
        }
    }

    fn next_challenge(&mut self) -> ChallengeContent {
        ChallengeContent {
            title: CatchPhase().fake::<String>(),
            description: Buzzword().fake::<String>(),
            rewards_total: 0,
            challenge_url: self.generator.gen_http_address(),
        }
    }

    fn next_fund(&mut self) -> FundContent {
        FundContent {
            goal: "How will we encourage developers and entrepreneurs to build Dapps and businesses on top of Cardano in the next 6 months?".to_string(),
            rewards_info: Sentence(3..5).fake::<String>()
        }
    }
}

pub struct ArbitraryValidVotingDataContent {
    generator: ArbitraryGenerator,
}

impl ArbitraryValidVotingDataContent {
    pub fn new() -> Self {
        Self {
            generator: ArbitraryGenerator::new(),
        }
    }
}
