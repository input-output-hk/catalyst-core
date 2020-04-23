use crate::{
    account::Identifier,
    certificate::{Proposal, VoteCast, VoteCastPayload, VotePlan, VotePlanId},
    date::BlockDate,
};
use std::collections::HashMap;
use thiserror::Error;

/// Manage the vote plan and the associated votes in the ledger
///
/// this structure manage the lifespan of the vote plan, the votes
/// casted and the associated parameters
pub struct VotePlanManager {
    id: VotePlanId,
    plan: VotePlan,

    proposal_managers: ProposalManagers,
}

pub struct ProposalManagers(Vec<ProposalManager>);

pub struct ProposalManager {
    votes_by_voters: HashMap<Identifier, VoteCastPayload>,
}

#[derive(Debug, Error)]
pub enum VoteError {
    #[error("Invalid vote plan, expected {expected}")]
    InvalidVotePlan {
        expected: VotePlanId,
        vote: VoteCast,
    },

    #[error("Invalid vote proposal, only {num_proposals} available in the vote plan")]
    InvalidVoteProposal {
        num_proposals: usize,
        vote: VoteCast,
    },
}

impl ProposalManager {
    fn new(_proposal: &Proposal) -> Self {
        Self {
            votes_by_voters: HashMap::new(),
        }
    }

    /// apply the given vote cast to the proposal
    ///
    /// if there is already a vote present for this proposal it will
    /// simply replace the previously set one
    ///
    pub fn vote(&mut self, identifier: Identifier, cast: VoteCast) -> Result<(), VoteError> {
        // we don't mind if we are replacing a vote
        let _ = self.votes_by_voters.insert(identifier, cast.into_payload());
        Ok(())
    }
}

impl ProposalManagers {
    fn new(plan: &VotePlan) -> Self {
        let proposal_managers = plan
            .proposals()
            .iter()
            .map(|proposal| ProposalManager::new(proposal))
            .collect();

        Self(proposal_managers)
    }

    /// attempt to apply the vote to one of the proposals
    ///
    /// if the proposal is not found this function will return an error.
    /// otherwise it will apply the vote. If the given identifier
    /// already had a vote, the previous vote will be discarded
    /// and only the new one will be kept
    pub fn vote(&mut self, identifier: Identifier, cast: VoteCast) -> Result<(), VoteError> {
        if let Some(proposal) = self.0.get_mut(cast.proposal_index() as usize) {
            proposal.vote(identifier, cast)
        } else {
            Err(VoteError::InvalidVoteProposal {
                num_proposals: self.0.len(),
                vote: cast,
            })
        }
    }
}

impl VotePlanManager {
    pub fn new(plan: VotePlan) -> Self {
        let id = plan.to_id();
        let proposal_managers = ProposalManagers::new(&plan);

        Self {
            id,
            plan,
            proposal_managers,
        }
    }

    pub fn id(&self) -> &VotePlanId {
        &self.id
    }

    pub fn plan(&self) -> &VotePlan {
        &self.plan
    }

    pub fn can_vote(&self, date: &BlockDate) -> bool {
        self.plan().can_vote(date)
    }

    pub fn can_committee(&self, date: &BlockDate) -> bool {
        self.plan().committee_time(date)
    }

    /// attempt to apply the vote to one of the proposals
    ///
    /// If the given identifier already had a vote, the previous vote will
    /// be discarded and only the new one will be kept.
    ///
    /// # errors
    ///
    /// * this function may fail if the proposal identifier is different
    /// * if the proposal index is not one one of the proposal listed
    ///
    pub fn vote(&mut self, identifier: Identifier, cast: VoteCast) -> Result<(), VoteError> {
        if cast.vote_plan() != self.id() {
            Err(VoteError::InvalidVotePlan {
                expected: self.id().clone(),
                vote: cast,
            })
        } else {
            self.proposal_managers.vote(identifier, cast)
        }
    }
}
