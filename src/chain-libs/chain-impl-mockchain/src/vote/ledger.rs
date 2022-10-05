use crate::ledger::token_distribution::TokenDistribution;
use crate::{
    account,
    certificate::{TallyProof, VoteAction, VoteCast, VotePlan, VotePlanId, VoteTally},
    date::BlockDate,
    ledger::governance::Governance,
    vote::{CommitteeId, PayloadType, VoteError, VotePlanManager},
};
use imhamt::{Hamt, InsertError, UpdateError};
use std::collections::{hash_map::DefaultHasher, HashSet};
use thiserror::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct VotePlanLedger {
    pub(crate) plans: Hamt<DefaultHasher, VotePlanId, VotePlanManager>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VotePlanLedgerError {
    #[error("cannot insert the vote plan {id}")]
    VotePlanInsertionError {
        id: VotePlanId,
        #[source]
        reason: InsertError,
    },

    #[error("cannot update the vote plan {id}")]
    VoteError {
        id: VotePlanId,
        #[source]
        reason: UpdateError<VoteError>,
    },

    #[error("Vote plan is set to finish in the passed ({vote_end}), current date {current_date}")]
    VotePlanVoteEndPassed {
        current_date: BlockDate,
        vote_end: BlockDate,
    },

    #[error("Vote plan already started ({vote_start}), current date {current_date}")]
    VotePlanVoteStartStartedAlready {
        current_date: BlockDate,
        vote_start: BlockDate,
    },

    #[error("Private vote plan must contain at least one committee member key")]
    VotePlanMissingCommitteeMemberKey,
}

impl VotePlanLedger {
    pub fn new() -> Self {
        Self { plans: Hamt::new() }
    }

    /// attempt to apply the vote to the appropriate Vote Proposal
    ///
    /// # errors
    ///
    /// can fail if:
    ///
    /// * the vote plan id does not exist;
    /// * the proposal's index does not exist;
    /// * it is no longer possible to vote (the date to vote expired)
    ///
    pub fn apply_vote(
        &self,
        block_date: BlockDate,
        identifier: account::Identifier,
        vote: VoteCast,
        token_distribution: TokenDistribution<()>,
    ) -> Result<Self, VotePlanLedgerError> {
        let id = vote.vote_plan().clone();

        let r = self.plans.update(&id, move |v| {
            v.vote(block_date, identifier, vote, token_distribution)
                .map(Some)
        });

        match r {
            Err(reason) => Err(VotePlanLedgerError::VoteError { reason, id }),
            Ok(plans) => Ok(Self { plans }),
        }
    }

    /// add the vote plan in a new `VotePlanLedger`
    ///
    /// the given `VotePlanLedger` is not modified and instead a new `VotePlanLedger` is
    /// returned. They share read-only memory.
    ///
    /// # errors if
    ///
    /// * the vote_plan is set to finished votes in the past
    /// * the vote_plan has already started
    ///
    #[must_use = "This function does not modify the object, the result contains the resulted new version of the vote plan ledger"]
    pub fn add_vote_plan(
        &self,
        current_date: BlockDate,
        vote_plan: VotePlan,
        committee: HashSet<CommitteeId>,
    ) -> Result<Self, VotePlanLedgerError> {
        if current_date > vote_plan.vote_end() {
            return Err(VotePlanLedgerError::VotePlanVoteEndPassed {
                current_date,
                vote_end: vote_plan.vote_end(),
            });
        }

        if current_date > vote_plan.vote_start() {
            return Err(VotePlanLedgerError::VotePlanVoteStartStartedAlready {
                current_date,
                vote_start: vote_plan.vote_start(),
            });
        }

        if let PayloadType::Private = vote_plan.payload_type() {
            if vote_plan.committee_public_keys().is_empty() {
                return Err(VotePlanLedgerError::VotePlanMissingCommitteeMemberKey);
            }
        }

        let id = vote_plan.to_id();
        let manager = VotePlanManager::new(vote_plan, committee);

        match self.plans.insert(id.clone(), manager) {
            Err(reason) => Err(VotePlanLedgerError::VotePlanInsertionError { id, reason }),
            Ok(plans) => Ok(Self { plans }),
        }
    }

    /// apply the committee result for the associated vote plan
    ///
    /// # Errors
    ///
    /// This function may fail:
    ///
    /// * if the Committee time has elapsed
    ///
    pub fn apply_committee_result<F>(
        &self,
        block_date: BlockDate,
        governance: &Governance,
        tally: &VoteTally,
        sig: TallyProof,
        token_distribution: TokenDistribution<()>,
        f: F,
    ) -> Result<Self, VotePlanLedgerError>
    where
        F: FnMut(&VoteAction),
    {
        let id = tally.id().clone();

        let committee_id = match sig {
            TallyProof::Public { id, .. } => id,
            TallyProof::Private { id, .. } => id,
        };
        let r = self.plans.update(&id, move |v| match sig {
            TallyProof::Public { .. } => v
                .public_tally(block_date, governance, committee_id, token_distribution, f)
                .map(Some),
            TallyProof::Private { .. } => {
                let shares = tally.tally_decrypted().unwrap();
                v.private_tally(
                    block_date,
                    shares,
                    governance,
                    committee_id,
                    token_distribution,
                    f,
                )
                .map(Some)
            }
        });

        match r {
            Err(reason) => Err(VotePlanLedgerError::VoteError { reason, id }),
            Ok(plans) => Ok(Self { plans }),
        }
    }
}

impl Default for VotePlanLedger {
    fn default() -> Self {
        Self::new()
    }
}
