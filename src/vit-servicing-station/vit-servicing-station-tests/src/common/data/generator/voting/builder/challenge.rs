use super::ProposalConfig;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

lazy_static! {
    static ref NEXT_CHALLENGE_ID: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Debug, Clone)]
pub struct ChallengeConfig {
    pub(crate) id: usize,
    pub(crate) proposals: VecDeque<ProposalConfig>,
    pub(crate) rewards_total: Option<u64>,
    pub(crate) proposers_rewards: Option<u64>,
}

impl Default for ChallengeConfig {
    fn default() -> Self {
        Self {
            id: NEXT_CHALLENGE_ID.fetch_add(1, Ordering::SeqCst),
            proposals: VecDeque::new(),
            rewards_total: None,
            proposers_rewards: None,
        }
    }
}

impl ChallengeConfig {
    pub fn proposals(mut self, proposals: Vec<ProposalConfig>) -> Self {
        for proposal in proposals.into_iter() {
            self = self.proposal(proposal);
        }
        self
    }

    pub fn proposal(mut self, mut proposal: ProposalConfig) -> Self {
        if proposal.challenge_id.is_none() {
            proposal.challenge_id = Some(self.id);
        }
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
