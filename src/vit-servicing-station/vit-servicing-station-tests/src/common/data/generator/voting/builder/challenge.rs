use super::ProposalBuilder;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct ChallengeBuilder {
    pub(crate) proposals: VecDeque<ProposalBuilder>,
    pub(crate) rewards_total: Option<u64>,
    pub(crate) proposers_rewards: Option<u64>,
}

impl ChallengeBuilder {
    pub fn proposals(mut self, proposals: Vec<ProposalBuilder>) -> Self {
        self.proposals = VecDeque::from(proposals);
        self
    }

    pub fn proposal(mut self, proposal: ProposalBuilder) -> Self {
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
