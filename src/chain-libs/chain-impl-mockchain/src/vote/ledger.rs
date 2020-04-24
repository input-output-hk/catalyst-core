use crate::{
    account::Identifier,
    certificate::{VoteCast, VotePlan, VotePlanId},
    date::BlockDate,
    vote::{VoteError, VotePlanManager},
};
use imhamt::{Hamt, InsertError, UpdateError};
use std::collections::{hash_map::DefaultHasher, BTreeMap};
use thiserror::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct VotePlanLedger {
    pub(crate) plans: Hamt<DefaultHasher, VotePlanId, (VotePlanManager, BlockDate)>,
    plans_by_end_date: BTreeMap<BlockDate, Vec<VotePlanId>>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VotePlanLedgerError {
    #[error("cannot insert the vote plan {id}: {reason:?}")]
    VotePlanInsertionError { id: VotePlanId, reason: InsertError },

    #[error("cannot insert the vote plan {id}: {reason:?}")]
    VoteError {
        id: VotePlanId,
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
}

impl VotePlanLedger {
    pub fn new() -> Self {
        Self {
            plans: Hamt::new(),
            plans_by_end_date: BTreeMap::new(),
        }
    }

    /// garbage collect the vote plans that should no longer be tracked
    /// and return the new state
    ///
    /// the block_date is supposed to be the current block date for the
    /// new state.
    ///
    /// This function is not to use lightly as this will remove VotePlans
    /// that are still interesting to track down:
    ///
    /// * we still need to publish the vote result;
    /// * we still need to distribute the rewards?
    ///
    pub fn gc(&self, block_date: &BlockDate) -> Self {
        let mut to_remove = self.plans_by_end_date.clone();
        let to_keep = to_remove.split_off(block_date);

        let mut plans = self.plans.clone();
        for ids in to_remove.values() {
            for id in ids {
                plans = match plans.remove(id) {
                    Err(remove_error) => {
                        // it should not be possible to happen
                        // if it does then there is something else
                        // going on, maybe in the add_vote function?
                        unreachable!(
                            "It should not be possible to fail to remove an entry: {:?}",
                            remove_error
                        )
                    }
                    Ok(plans) => plans,
                };
            }
        }

        Self {
            plans,
            plans_by_end_date: to_keep,
        }
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
        block_date: &BlockDate,
        identifier: Identifier,
        vote: VoteCast,
    ) -> Result<Self, VotePlanLedgerError> {
        let id = vote.vote_plan().clone();

        let r = self.plans.update(&id, move |(v, _)| {
            v.vote(block_date, identifier, vote)
                .map(|v| Some((v, block_date.clone())))
        });

        match r {
            Err(reason) => Err(VotePlanLedgerError::VoteError { reason, id }),
            Ok(plans) => Ok(Self {
                plans,
                plans_by_end_date: self.plans_by_end_date.clone(),
            }),
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

        let id = vote_plan.to_id();
        let end_date = vote_plan.committee_end();
        let manager = VotePlanManager::new(vote_plan);

        match self.plans.insert(id.clone(), (manager, end_date.clone())) {
            Err(reason) => Err(VotePlanLedgerError::VotePlanInsertionError { id, reason }),
            Ok(plans) => {
                let mut plans_by_end_date = self.plans_by_end_date.clone();
                plans_by_end_date
                    .entry(end_date)
                    .or_insert(Vec::default())
                    .push(id);
                Ok(Self {
                    plans,
                    plans_by_end_date,
                })
            }
        }
    }

    /// apply the committee result for the associated vote plan
    ///
    /// TODO: this function is not implemented
    pub fn apply_committee_result(&self) -> Self {
        todo!()
    }
}

impl Default for VotePlanLedger {
    fn default() -> Self {
        Self::new()
    }
}
