use super::ProposalConfig;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct ChallengeConfig {
    pub(crate) proposals: VecDeque<ProposalConfig>,
    pub(crate) rewards_total: Option<u64>,
    pub(crate) proposers_rewards: Option<u64>,
}

impl ChallengeConfig {
    pub fn proposals(mut self, proposals: Vec<ProposalConfig>) -> Self {
        self.proposals = VecDeque::from(proposals);
        self
    }

    pub fn proposals_len(&self) -> usize {
        self.proposals.len()
    }

    pub fn proposal(mut self, proposal: ProposalConfig) -> Self {
        self.proposals.push_back(proposal);
        self
    }

    pub fn rewards_total(mut self, rewards_total: u64) -> Self {
        self.rewards_total = Some(rewards_total);
        self
    }

    pub fn proposers_rewards(mut self, proposers_rewards: u64) -> Self {
        self.proposers_rewards = Some(proposers_rewards);
        self
    }
}
