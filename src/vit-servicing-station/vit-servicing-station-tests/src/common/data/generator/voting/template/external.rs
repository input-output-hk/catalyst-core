use super::{ChallengeTemplate, FundTemplate, ProposalTemplate, ValidVotingTemplateGenerator};
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
}

#[derive(Clone)]
pub struct ExternalValidVotingTemplateGenerator {
    proposals: LinkedList<ProposalTemplate>,
    challenges: LinkedList<ChallengeTemplate>,
    funds: LinkedList<FundTemplate>,
}

impl ExternalValidVotingTemplateGenerator {
    pub fn new(
        proposals: PathBuf,
        challenges: PathBuf,
        funds: PathBuf,
    ) -> Result<Self, TemplateLoadError> {
        Ok(Self {
            proposals: parse_proposals(proposals)?,
            challenges: parse_challenges(challenges)?,
            funds: parse_funds(funds)?,
        })
    }

    pub fn proposals_count(&self) -> usize {
        self.proposals.len()
    }

    pub fn challenges_count(&self) -> usize {
        self.challenges.len()
    }
}

pub fn parse_proposals(
    proposals: PathBuf,
) -> Result<LinkedList<ProposalTemplate>, TemplateLoadError> {
    serde_json::from_str(&jortestkit::file::read_file(&proposals))
        .map_err(|err| TemplateLoadError::ProposalTemplate(err.to_string()))
}

pub fn parse_challenges(
    challenges: PathBuf,
) -> Result<LinkedList<ChallengeTemplate>, TemplateLoadError> {
    serde_json::from_str(&jortestkit::file::read_file(&challenges))
        .map_err(|err| TemplateLoadError::ChallengeTemplate(err.to_string()))
}

pub fn parse_funds(funds: PathBuf) -> Result<LinkedList<FundTemplate>, TemplateLoadError> {
    serde_json::from_str(&jortestkit::file::read_file(&funds))
        .map_err(|err| TemplateLoadError::FundTemplate(err.to_string()))
}

#[derive(Debug, Error)]
pub enum TemplateLoadError {
    #[error("cannot parse proposals, due to {0}")]
    ProposalTemplate(String),
    #[error("cannot parse challenges, due to: {0}")]
    ChallengeTemplate(String),
    #[error("cannot parse funds, due to: {0}")]
    FundTemplate(String),
}
