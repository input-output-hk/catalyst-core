use super::{
    ChallengeTemplate, FundTemplate, ProposalTemplate, ReviewTemplate, ValidVotingTemplateGenerator,
};
use std::{collections::LinkedList, path::PathBuf};
use thiserror::Error;

impl ValidVotingTemplateGenerator for ExternalValidVotingTemplateGenerator {
    fn next_proposal(&mut self) -> ProposalTemplate {
        self.proposals
            .pop_front()
            .unwrap_or_else(|| panic!("no more proposals"))
    }

    fn next_challenge(&mut self) -> ChallengeTemplate {
        self.challenges
            .pop_front()
            .unwrap_or_else(|| panic!("no more challenges"))
    }

    fn next_fund(&mut self) -> FundTemplate {
        self.funds
            .pop_front()
            .unwrap_or_else(|| panic!("no more funds"))
    }

    fn next_review(&mut self) -> ReviewTemplate {
        self.reviews
            .pop_front()
            .unwrap_or_else(|| panic!("no more reviews"))
    }
}

#[derive(Clone)]
pub struct ExternalValidVotingTemplateGenerator {
    proposals: LinkedList<ProposalTemplate>,
    challenges: LinkedList<ChallengeTemplate>,
    funds: LinkedList<FundTemplate>,
    reviews: LinkedList<ReviewTemplate>,
}

impl ExternalValidVotingTemplateGenerator {
    pub fn new(
        proposals: PathBuf,
        challenges: PathBuf,
        funds: PathBuf,
        reviews: PathBuf,
    ) -> Result<Self, TemplateLoad> {
        Ok(Self {
            proposals: parse_proposals(proposals)?,
            challenges: parse_challenges(challenges)?,
            funds: parse_funds(funds)?,
            reviews: parse_reviews(reviews)?,
        })
    }

    pub fn proposals_count(&self) -> usize {
        self.proposals.len()
    }

    pub fn challenges_count(&self) -> usize {
        self.challenges.len()
    }
}

pub fn parse_proposals(proposals: PathBuf) -> Result<LinkedList<ProposalTemplate>, TemplateLoad> {
    serde_json::from_str(&std::fs::read_to_string(&proposals)?)
        .map_err(|err| TemplateLoad::Proposal(err.to_string()))
}

pub fn parse_challenges(
    challenges: PathBuf,
) -> Result<LinkedList<ChallengeTemplate>, TemplateLoad> {
    serde_json::from_str(&std::fs::read_to_string(&challenges)?)
        .map_err(|err| TemplateLoad::Challenge(err.to_string()))
}

pub fn parse_funds(funds: PathBuf) -> Result<LinkedList<FundTemplate>, TemplateLoad> {
    serde_json::from_str(&std::fs::read_to_string(&funds)?)
        .map_err(|err| TemplateLoad::Fund(err.to_string()))
}

pub fn parse_reviews(reviews: PathBuf) -> Result<LinkedList<ReviewTemplate>, TemplateLoad> {
    serde_json::from_str(&std::fs::read_to_string(&reviews)?)
        .map_err(|err| TemplateLoad::Review(err.to_string()))
}

#[derive(Debug, Error)]
pub enum TemplateLoad {
    #[error("cannot parse proposals, due to {0}")]
    Proposal(String),
    #[error("cannot parse challenges, due to: {0}")]
    Challenge(String),
    #[error("cannot parse funds, due to: {0}")]
    Fund(String),
    #[error("cannot parse reviews, due to: {0}")]
    Review(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
