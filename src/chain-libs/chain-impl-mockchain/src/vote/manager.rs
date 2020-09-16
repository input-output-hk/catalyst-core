use crate::{
    certificate::{Proposal, TallyProof, VoteAction, VoteCast, VotePlan, VotePlanId},
    date::BlockDate,
    ledger::governance::{Governance, GovernanceAcceptanceCriteria},
    rewards::Ratio,
    stake::{Stake, StakeControl},
    transaction::UnspecifiedAccountIdentifier,
    vote::{self, CommitteeId, Options, Tally, TallyResult, VotePlanStatus, VoteProposalStatus},
};
use imhamt::Hamt;
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    num::NonZeroU64,
    sync::Arc,
};
use thiserror::Error;

/// Manage the vote plan and the associated votes in the ledger
///
/// this structure manage the lifespan of the vote plan, the votes
/// casted and the associated parameters
#[derive(Clone, PartialEq, Eq)]
pub struct VotePlanManager {
    id: VotePlanId,
    plan: Arc<VotePlan>,
    committee: Arc<HashSet<CommitteeId>>,

    proposal_managers: ProposalManagers,
}

#[derive(Clone, PartialEq, Eq)]
struct ProposalManagers(Vec<ProposalManager>);

#[derive(Clone, PartialEq, Eq)]
struct ProposalManager {
    votes_by_voters: Hamt<DefaultHasher, UnspecifiedAccountIdentifier, vote::Payload>,
    options: Options,
    tally: Option<Tally>,
    action: VoteAction,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VoteError {
    #[error("Invalid vote plan, expected {expected}")]
    InvalidVotePlan {
        expected: VotePlanId,
        vote: VoteCast,
    },

    #[error("It is not possible to vote at the moment for the proposals, time to vote is between {start} to {end}.")]
    NotVoteTime {
        start: BlockDate,
        end: BlockDate,
        vote: VoteCast,
    },

    #[error("Invalid vote proposal, only {num_proposals} available in the vote plan")]
    InvalidVoteProposal {
        num_proposals: usize,
        vote: VoteCast,
    },

    #[error("{received:?} is not the expected payload type, expected {expected:?}")]
    InvalidPayloadType {
        received: vote::PayloadType,
        expected: vote::PayloadType,
    },

    #[error("It is not possible to tally the votes for the proposals, time to tally the votes is between {start} to {end}.")]
    NotCommitteeTime { start: BlockDate, end: BlockDate },

    #[error("Unexpected TallyProof's public ID, expected one of the committee")]
    InvalidTallyCommittee,

    #[error("Cannot tally votes")]
    CannotTallyVotes {
        #[source]
        #[from]
        source: vote::TallyError,
    },
}

impl ProposalManager {
    /// construct a `ProposalManager` to track down the votes associated to this
    /// proposal.
    ///
    /// the proposal is passed on as parameter so we could add some form
    /// of verification in the future about the content of the vote (if
    /// possible : ZK is not necessarily allowing this).
    ///
    fn new(proposal: &Proposal) -> Self {
        Self {
            votes_by_voters: Hamt::new(),
            options: proposal.options().clone(),
            tally: None,
            action: proposal.action().clone(),
        }
    }

    /// apply the given vote cast to the proposal
    ///
    /// if there is already a vote present for this proposal it will
    /// simply replace the previously set one
    ///
    #[must_use = "Add the vote in a new ProposalManager, does not modify self"]
    pub fn vote(
        &self,
        identifier: UnspecifiedAccountIdentifier,
        cast: VoteCast,
    ) -> Result<Self, VoteError> {
        let payload = cast.into_payload();

        // we don't mind if we are replacing a vote
        let votes_by_voters =
            self.votes_by_voters
                .insert_or_update_simple(identifier, payload.clone(), |_| Some(payload));
        Ok(Self {
            votes_by_voters,
            tally: self.tally.clone(),
            options: self.options.clone(),
            action: self.action.clone(),
        })
    }

    #[must_use = "Compute the PublicTally in a new ProposalManager, does not modify self"]
    pub fn public_tally<F>(
        &self,
        stake: &StakeControl,
        governance: &Governance,
        f: &mut F,
    ) -> Result<Self, VoteError>
    where
        F: FnMut(&VoteAction),
    {
        let mut results = TallyResult::new(self.options.clone());

        for (id, payload) in self.votes_by_voters.iter() {
            if let Some(account_id) = id.to_single_account() {
                if let Some(stake) = stake.by(&account_id) {
                    match payload {
                        vote::Payload::Public { choice } => {
                            results.add_vote(*choice, stake)?;
                        }
                    }
                }
            }
        }

        if self.check(stake.assigned(), governance, &results) {
            f(&self.action)
        }

        Ok(Self {
            votes_by_voters: self.votes_by_voters.clone(),
            options: self.options.clone(),
            tally: Some(Tally::new_public(results)),
            action: self.action.clone(),
        })
    }

    fn check(&self, total: Stake, governance: &Governance, results: &TallyResult) -> bool {
        match &self.action {
            VoteAction::OffChain => false,
            VoteAction::Treasury { action } => {
                let t = action.to_type();
                let acceptance = governance.treasury.acceptance_criteria_for(t);

                self.check_governance_criteria(total, acceptance, results)
            }
            VoteAction::Parameters { action } => {
                let t = action.to_type();
                let acceptance = governance.parameters.acceptance_criteria_for(t);

                self.check_governance_criteria(total, acceptance, results)
            }
        }
    }

    fn check_governance_criteria(
        &self,
        total: Stake,
        acceptance: &GovernanceAcceptanceCriteria,
        results: &TallyResult,
    ) -> bool {
        let total = if let Some(t) = NonZeroU64::new(total.into()) {
            t
        } else {
            return false;
        };
        let participation = if let Some(p) = NonZeroU64::new(results.participation().into()) {
            p
        } else {
            return false;
        };
        let favorable: u64 = if let Some(weight) = results
            .results()
            .get(acceptance.favorable.as_byte() as usize)
        {
            (*weight).into()
        } else {
            return false;
        };
        let non_blanks = if let Some(weight) = results
            .results()
            .get(acceptance.rejection.as_byte() as usize)
        {
            let v: u64 = (*weight).into();
            if let Some(v) = NonZeroU64::new(v + favorable) {
                v
            } else {
                return false;
            }
        } else {
            return false;
        };

        let ratio_favorable = Ratio {
            numerator: favorable,
            denominator: non_blanks,
        };

        let ratio_participation = Ratio {
            numerator: participation.into(),
            denominator: total,
        };

        if let Some(criteria) = acceptance.minimum_stake_participation {
            if ratio_participation <= criteria {
                return false;
            }
        }

        if let Some(criteria) = acceptance.minimum_approval {
            if ratio_favorable <= criteria {
                return false;
            }
        }

        true
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
    pub fn vote(
        &self,
        identifier: UnspecifiedAccountIdentifier,
        cast: VoteCast,
    ) -> Result<Self, VoteError> {
        let proposal_index = cast.proposal_index() as usize;
        if let Some(manager) = self.0.get(proposal_index) {
            let updated_manager = manager.vote(identifier, cast)?;

            // only clone the array if it does make sens to do so:
            //
            // * the index exist
            // * updated_manager succeed
            let mut updated = self.clone();

            // not unsafe to call this function since we already know this
            // `proposal_index` already exist in the array
            unsafe { *updated.0.get_unchecked_mut(proposal_index) = updated_manager };

            Ok(updated)
        } else {
            Err(VoteError::InvalidVoteProposal {
                num_proposals: self.0.len(),
                vote: cast,
            })
        }
    }

    pub fn public_tally<F>(
        &self,
        stake: &StakeControl,
        governance: &Governance,
        f: &mut F,
    ) -> Result<Self, VoteError>
    where
        F: FnMut(&VoteAction),
    {
        let mut proposals = Vec::with_capacity(self.0.len());
        for proposal in self.0.iter() {
            proposals.push(proposal.public_tally(stake, governance, f)?);
        }

        Ok(Self(proposals))
    }
}

impl VotePlanManager {
    pub fn new(plan: VotePlan, committee: HashSet<CommitteeId>) -> Self {
        let id = plan.to_id();
        let proposal_managers = ProposalManagers::new(&plan);

        Self {
            id,
            plan: Arc::new(plan),
            proposal_managers,
            committee: Arc::new(committee),
        }
    }

    pub fn id(&self) -> &VotePlanId {
        &self.id
    }

    pub fn plan(&self) -> &VotePlan {
        &self.plan
    }

    pub fn statuses(&self) -> VotePlanStatus {
        let proposals = self
            .plan()
            .proposals()
            .iter()
            .zip(self.proposal_managers.0.iter())
            .enumerate()
            .map(|(index, (proposal, manager))| VoteProposalStatus {
                index: index as u8,
                proposal_id: proposal.external_id().clone(),
                options: proposal.options().clone(),
                tally: manager.tally.clone(),
                votes: manager.votes_by_voters.clone(),
            })
            .collect();

        VotePlanStatus {
            id: self.id.clone(),
            payload: self.plan().payload_type(),
            vote_start: self.plan().vote_start(),
            vote_end: self.plan().vote_end(),
            committee_end: self.plan().committee_end(),
            proposals,
        }
    }

    pub fn can_vote(&self, date: BlockDate) -> bool {
        self.plan().can_vote(date)
    }

    pub fn can_committee(&self, date: BlockDate) -> bool {
        self.plan().committee_time(date)
    }

    pub fn committee_set(&self) -> &HashSet<CommitteeId> {
        &self.committee
    }

    /// return true if the vote plan has elapsed i.e. the vote is
    /// no longer interesting to track in the ledger and it can be
    /// GCed.
    pub fn vote_plan_elapsed(&self, date: BlockDate) -> bool {
        self.plan().committee_end() < date
    }

    fn valid_committee(&self, sig: TallyProof) -> bool {
        match sig {
            TallyProof::Public { id, signature: _ } => self.committee_set().contains(&id),
        }
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
    /// * if the block_date show it is no longer valid to cast a vote for any
    ///   of the managed proposals
    /// * if the payload type of the vote is not the expected one
    ///
    pub fn vote(
        &self,
        block_date: BlockDate,
        identifier: UnspecifiedAccountIdentifier,
        cast: VoteCast,
    ) -> Result<Self, VoteError> {
        if cast.vote_plan() != self.id() {
            Err(VoteError::InvalidVotePlan {
                expected: self.id().clone(),
                vote: cast,
            })
        } else if !self.can_vote(block_date) {
            Err(VoteError::NotVoteTime {
                start: self.plan().vote_start(),
                end: self.plan().vote_end(),
                vote: cast,
            })
        } else if self.plan().payload_type() != cast.payload().payload_type() {
            Err(VoteError::InvalidPayloadType {
                expected: self.plan().payload_type(),
                received: cast.payload().payload_type(),
            })
        } else {
            let proposal_managers = self.proposal_managers.vote(identifier, cast)?;

            Ok(Self {
                proposal_managers,
                plan: Arc::clone(&self.plan),
                id: self.id.clone(),
                committee: Arc::clone(&self.committee),
            })
        }
    }

    pub fn tally<F>(
        &self,
        block_date: BlockDate,
        stake: &StakeControl,
        governance: &Governance,
        sig: TallyProof,
        f: &mut F,
    ) -> Result<Self, VoteError>
    where
        F: FnMut(&VoteAction),
    {
        if !self.can_committee(block_date) {
            Err(VoteError::NotCommitteeTime {
                start: self.plan().committee_start(),
                end: self.plan().committee_end(),
            })
        } else if !self.valid_committee(sig) {
            Err(VoteError::InvalidTallyCommittee)
        } else {
            let proposal_managers = match self.plan().payload_type() {
                vote::PayloadType::Public => {
                    self.proposal_managers.public_tally(&stake, governance, f)?
                }
            };

            Ok(Self {
                proposal_managers,
                plan: Arc::clone(&self.plan),
                id: self.id.clone(),
                committee: Arc::clone(&self.committee),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockDate;
    use crate::testing::{TestGen, VoteTestGen};
    use chain_core::property::BlockDate as BlockDateProp;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[test]
    pub fn proposal_manager_insert_vote() {
        let vote_plan = VoteTestGen::vote_plan();
        let vote_cast_payload = vote::Payload::public(vote::Choice::new(1));
        let vote_cast = VoteCast::new(vote_plan.to_id(), 0, vote_cast_payload.clone());

        let mut proposal_manager = ProposalManager::new(vote_plan.proposals().get(0).unwrap());

        let identifier = TestGen::unspecified_account_identifier();
        proposal_manager = proposal_manager
            .vote(identifier.clone(), vote_cast)
            .unwrap();

        let (_, actual_vote_cast_payload) = proposal_manager
            .votes_by_voters
            .iter()
            .find(|(x, _y)| **x == identifier)
            .unwrap();
        assert_eq!(*actual_vote_cast_payload, vote_cast_payload);
    }

    #[test]
    pub fn proposal_manager_replace_vote() {
        let vote_plan = VoteTestGen::vote_plan();
        let first_vote_cast_payload = VoteTestGen::vote_cast_payload();
        let second_vote_cast_payload = VoteTestGen::vote_cast_payload();

        let first_vote_cast = VoteCast::new(vote_plan.to_id(), 0, first_vote_cast_payload);
        let second_vote_cast =
            VoteCast::new(vote_plan.to_id(), 0, second_vote_cast_payload.clone());

        let mut proposal_manager = ProposalManager::new(vote_plan.proposals().get(0).unwrap());

        let identifier = TestGen::unspecified_account_identifier();
        proposal_manager = proposal_manager
            .vote(identifier.clone(), first_vote_cast)
            .unwrap();
        proposal_manager = proposal_manager
            .vote(identifier.clone(), second_vote_cast)
            .unwrap();

        let (_, actual_vote_cast_payload) = proposal_manager
            .votes_by_voters
            .iter()
            .find(|(x, _y)| **x == identifier)
            .unwrap();
        assert_eq!(*actual_vote_cast_payload, second_vote_cast_payload);
    }

    const CENT: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(100) };
    use crate::certificate::Proposals;
    use crate::ledger::governance::{ParametersGovernance, ParametersGovernanceAction};
    use crate::ledger::governance::{TreasuryGovernance, TreasuryGovernanceAction};
    use crate::value::Value;
    use crate::vote::Choice;

    #[test]
    pub fn vote_plan_manager_statuses() {
        let proposals = VoteTestGen::proposals(3);

        let vote_plan = VotePlan::new(
            BlockDate::from_epoch_slot_id(1, 0),
            BlockDate::from_epoch_slot_id(2, 0),
            BlockDate::from_epoch_slot_id(3, 0),
            proposals,
            vote::PayloadType::Public,
        );

        let vote_plan_manager = VotePlanManager::new(vote_plan.clone(), HashSet::new());

        let status = vote_plan_manager.statuses();

        assert_eq!(status.id, vote_plan.to_id());
        assert_eq!(status.payload, vote_plan.payload_type());
        assert_eq!(status.vote_start, vote_plan.vote_start());
        assert_eq!(status.vote_end, vote_plan.vote_end());
        assert_eq!(status.committee_end, vote_plan.committee_end());
        assert_eq!(status.proposals.len(), 3);

        assert_eq!(vote_plan_manager.committee_set().len(), 0);
    }

    use crate::fee::LinearFee;
    use crate::fragment::Fragment;
    use crate::testing::build_vote_tally_cert;
    use crate::testing::data::Wallet;
    use crate::testing::TestTxCertBuilder;

    #[test]
    pub fn vote_plan_manager_correct_tally() {
        let blank = Choice::new(0);
        let favorable = Choice::new(1);
        let rejection = Choice::new(2);
        let committee = Wallet::from_value(Value(100));
        let proposals = VoteTestGen::proposals_with_action(
            VoteAction::Treasury {
                action: TreasuryGovernanceAction::TransferToRewards { value: Value(30) },
            },
            3,
        );

        let vote_plan = VotePlan::new(
            BlockDate::from_epoch_slot_id(1, 0),
            BlockDate::from_epoch_slot_id(2, 0),
            BlockDate::from_epoch_slot_id(3, 0),
            proposals,
            vote::PayloadType::Public,
        );

        let mut committee_ids = HashSet::new();
        committee_ids.insert(committee.public_key().into());
        let mut vote_plan_manager = VotePlanManager::new(vote_plan.clone(), committee_ids);

        let governance = governance_50_percent(&blank, &favorable, &rejection);
        let mut stake_controlled = StakeControl::new();
        stake_controlled = stake_controlled.add_to(committee.public_key().into(), Stake(51));
        //    stake_controlled = stake_controlled.add_unassigned(Stake(49));

        let vote_block_date = BlockDate {
            epoch: 1,
            slot_id: 10,
        };

        let vote_cast = VoteCast::new(
            vote_plan.to_id(),
            0,
            VoteTestGen::vote_cast_payload_for(&favorable),
        );

        vote_plan_manager = vote_plan_manager
            .vote(
                vote_block_date,
                UnspecifiedAccountIdentifier::from_single_account(committee.public_key().into()),
                vote_cast,
            )
            .unwrap();

        let tally_proof = get_tally_proof(&committee, vote_plan.to_id());

        let block_date = BlockDate {
            epoch: 2,
            slot_id: 10,
        };

        let mut action_hit = false;
        vote_plan_manager
            .tally(
                block_date,
                &stake_controlled,
                &governance,
                tally_proof,
                &mut |_| action_hit = true,
            )
            .unwrap();
        assert!(action_hit)
    }

    #[test]
    pub fn vote_plan_manager_tally_invalid_committee() {
        let blank = Choice::new(0);
        let favorable = Choice::new(1);
        let rejection = Choice::new(2);
        let committee = Wallet::from_value(Value(100));
        let proposals = VoteTestGen::proposals(3);

        let vote_plan = VotePlan::new(
            BlockDate::from_epoch_slot_id(1, 0),
            BlockDate::from_epoch_slot_id(2, 0),
            BlockDate::from_epoch_slot_id(3, 0),
            proposals,
            vote::PayloadType::Public,
        );

        let mut committee_ids = HashSet::new();
        committee_ids.insert(TestGen::public_key().into());
        let vote_plan_manager = VotePlanManager::new(vote_plan.clone(), committee_ids);

        let governance = governance_50_percent(&blank, &favorable, &rejection);
        let mut stake_controlled = StakeControl::new();
        stake_controlled = stake_controlled.add_to(committee.public_key().into(), Stake(51));
        stake_controlled = stake_controlled.add_unassigned(Stake(49));

        let tally_proof = get_tally_proof(&committee, vote_plan.to_id());

        let block_date = BlockDate {
            epoch: 2,
            slot_id: 10,
        };

        //invalid committee
        assert_eq!(
            VoteError::InvalidTallyCommittee,
            vote_plan_manager
                .tally(
                    block_date,
                    &stake_controlled,
                    &governance,
                    tally_proof,
                    &mut |_| ()
                )
                .err()
                .unwrap()
        );
    }

    #[test]
    pub fn vote_plan_manager_tally_invalid_date() {
        let blank = Choice::new(0);
        let favorable = Choice::new(1);
        let rejection = Choice::new(2);
        let committee = Wallet::from_value(Value(100));
        let proposals = VoteTestGen::proposals(3);

        let vote_plan = VotePlan::new(
            BlockDate::from_epoch_slot_id(1, 0),
            BlockDate::from_epoch_slot_id(2, 0),
            BlockDate::from_epoch_slot_id(3, 0),
            proposals,
            vote::PayloadType::Public,
        );

        let mut committee_ids = HashSet::new();
        committee_ids.insert(committee.public_key().into());
        let vote_plan_manager = VotePlanManager::new(vote_plan.clone(), committee_ids);

        let governance = governance_50_percent(&blank, &favorable, &rejection);
        let mut stake_controlled = StakeControl::new();
        stake_controlled = stake_controlled.add_to(committee.public_key().into(), Stake(51));
        stake_controlled = stake_controlled.add_unassigned(Stake(49));

        let tally_proof = get_tally_proof(&committee, vote_plan.to_id());

        let invalid_block_date = BlockDate {
            epoch: 0,
            slot_id: 10,
        };

        //not in committee time
        assert_eq!(
            VoteError::NotCommitteeTime {
                start: vote_plan.committee_start(),
                end: vote_plan.committee_end()
            },
            vote_plan_manager
                .tally(
                    invalid_block_date,
                    &stake_controlled,
                    &governance,
                    tally_proof,
                    &mut |_| ()
                )
                .err()
                .unwrap()
        );
    }

    fn get_tally_proof(wallet: &Wallet, id: VotePlanId) -> TallyProof {
        let certificate = build_vote_tally_cert(id);
        let fragment = TestTxCertBuilder::new(TestGen::hash(), LinearFee::new(0, 0, 0))
            .make_transaction(&[&wallet], &certificate);

        match fragment {
            Fragment::VoteTally(tx) => {
                let tx = tx.as_slice();
                tx.payload_auth().into_payload_auth()
            }
            _ => unreachable!(),
        }
    }

    #[test]
    pub fn proposal_manager_vote_tally() {
        let blank = Choice::new(0);
        let favorable = Choice::new(1);
        let rejection = Choice::new(2);

        let mut proposals = Proposals::new();
        let _ = proposals.push(VoteTestGen::proposal_with_action(VoteAction::Treasury {
            action: TreasuryGovernanceAction::TransferToRewards { value: Value(30) },
        }));
        let _ = proposals.push(VoteTestGen::proposal_with_action(VoteAction::Parameters {
            action: ParametersGovernanceAction::RewardAdd { value: Value(30) },
        }));

        let vote_plan = VotePlan::new(
            BlockDate::from_epoch_slot_id(1, 0),
            BlockDate::from_epoch_slot_id(2, 0),
            BlockDate::from_epoch_slot_id(3, 0),
            proposals,
            vote::PayloadType::Public,
        );

        let mut first_proposal_manager =
            ProposalManager::new(vote_plan.proposals().get(0).unwrap());
        let mut second_proposal_manager =
            ProposalManager::new(vote_plan.proposals().get(1).unwrap());

        let identifier = TestGen::unspecified_account_identifier();

        let first_vote_cast = VoteCast::new(
            vote_plan.to_id(),
            0,
            VoteTestGen::vote_cast_payload_for(&favorable),
        );
        first_proposal_manager = first_proposal_manager
            .vote(identifier.clone(), first_vote_cast.clone())
            .unwrap();

        let second_vote_cast = VoteCast::new(
            vote_plan.to_id(),
            1,
            VoteTestGen::vote_cast_payload_for(&favorable),
        );
        second_proposal_manager = second_proposal_manager
            .vote(identifier.clone(), second_vote_cast.clone())
            .unwrap();

        let mut stake_controlled = StakeControl::new();
        stake_controlled =
            stake_controlled.add_to(identifier.to_single_account().unwrap(), Stake(51));
        stake_controlled = stake_controlled.add_unassigned(Stake(49));

        let proposals = ProposalManagers::new(&vote_plan);
        let _ = proposals.vote(identifier.clone(), first_vote_cast.clone());
        let _ = proposals.vote(identifier.clone(), second_vote_cast.clone());

        let governance = governance_50_percent(&blank, &favorable, &rejection);
        proposals_vote_tally_succesful(&proposals, &stake_controlled, &governance);
        vote_tally_succesful(&first_proposal_manager, &stake_controlled, &governance);
        vote_tally_succesful(&second_proposal_manager, &stake_controlled, &governance);
    }

    fn governance_50_percent(blank: &Choice, favorable: &Choice, rejection: &Choice) -> Governance {
        let gov_acceptance_criteria = GovernanceAcceptanceCriteria {
            minimum_stake_participation: Some(Ratio {
                numerator: 50,
                denominator: CENT,
            }),
            minimum_approval: Some(Ratio {
                numerator: 50,
                denominator: CENT,
            }),
            blank: blank.clone(),
            favorable: favorable.clone(),
            rejection: rejection.clone(),
            options: Options::new_length(3).expect("3 valid choices possible"),
        };

        let mut treasury_governance = TreasuryGovernance::new();
        treasury_governance.set_default_acceptance_criteria(gov_acceptance_criteria.clone());

        let mut parameters_governance = ParametersGovernance::new();
        parameters_governance.set_default_acceptance_criteria(gov_acceptance_criteria.clone());
        Governance {
            treasury: treasury_governance,
            parameters: parameters_governance,
        }
    }

    fn proposals_vote_tally_succesful(
        proposal_managers: &ProposalManagers,
        stake_controlled: &StakeControl,
        governance: &Governance,
    ) {
        let mut vote_action_hit = false;
        proposal_managers
            .public_tally(&stake_controlled, &governance, &mut |_vote_action| {
                vote_action_hit = true;
            })
            .unwrap();
    }

    fn vote_tally_succesful(
        proposal_manager: &ProposalManager,
        stake_controlled: &StakeControl,
        governance: &Governance,
    ) {
        let mut vote_action_hit = false;
        proposal_manager
            .public_tally(&stake_controlled, &governance, &mut |_vote_action| {
                vote_action_hit = true;
            })
            .unwrap();

        assert!(vote_action_hit);
    }

    #[test]
    pub fn proposal_managers_many_votes() {
        let vote_plan = VoteTestGen::vote_plan_with_proposals(2);
        let first_vote_cast_payload = VoteTestGen::vote_cast_payload();
        let second_vote_cast_payload = VoteTestGen::vote_cast_payload();

        let first_vote_cast = VoteCast::new(vote_plan.to_id(), 0, first_vote_cast_payload.clone());
        let second_vote_cast =
            VoteCast::new(vote_plan.to_id(), 1, second_vote_cast_payload.clone());

        let mut proposal_managers = ProposalManagers::new(&vote_plan);

        let identifier = TestGen::unspecified_account_identifier();
        proposal_managers = proposal_managers
            .vote(identifier.clone(), first_vote_cast)
            .unwrap();
        proposal_managers = proposal_managers
            .vote(identifier.clone(), second_vote_cast)
            .unwrap();

        let (_, actual_vote_cast_payload) = proposal_managers
            .0
            .get(0)
            .unwrap()
            .votes_by_voters
            .iter()
            .find(|(x, _y)| **x == identifier)
            .unwrap();
        assert_eq!(*actual_vote_cast_payload, first_vote_cast_payload);

        let (_, actual_vote_cast_payload) = proposal_managers
            .0
            .get(1)
            .unwrap()
            .votes_by_voters
            .iter()
            .find(|(x, _y)| **x == identifier)
            .unwrap();
        assert_eq!(*actual_vote_cast_payload, second_vote_cast_payload);
    }

    #[test]
    pub fn vote_for_nonexisting_proposal() {
        let vote_plan = VoteTestGen::vote_plan_with_proposals(1);
        let vote_cast = VoteCast::new(vote_plan.to_id(), 2, VoteTestGen::vote_cast_payload());

        let proposal_managers = ProposalManagers::new(&vote_plan);
        assert!(proposal_managers
            .vote(TestGen::unspecified_account_identifier(), vote_cast)
            .is_err());
    }

    #[test]
    pub fn proposal_managers_update_vote() {
        let vote_plan = VoteTestGen::vote_plan_with_proposals(2);
        let first_vote_cast_payload = VoteTestGen::vote_cast_payload();
        let second_vote_cast_payload = VoteTestGen::vote_cast_payload();

        let first_vote_cast = VoteCast::new(vote_plan.to_id(), 0, first_vote_cast_payload);
        let second_vote_cast =
            VoteCast::new(vote_plan.to_id(), 0, second_vote_cast_payload.clone());

        let mut proposal_managers = ProposalManagers::new(&vote_plan);

        let identifier = TestGen::unspecified_account_identifier();
        proposal_managers = proposal_managers
            .vote(identifier.clone(), first_vote_cast)
            .unwrap();
        proposal_managers = proposal_managers
            .vote(identifier.clone(), second_vote_cast)
            .unwrap();

        let (_, actual_vote_cast_payload) = proposal_managers
            .0
            .get(0)
            .unwrap()
            .votes_by_voters
            .iter()
            .find(|(x, _y)| **x == identifier)
            .unwrap();
        assert_eq!(*actual_vote_cast_payload, second_vote_cast_payload);
    }

    #[quickcheck]
    pub fn vote_plan_manager_can_vote(vote_plan: VotePlan, date: BlockDate) -> TestResult {
        let vote_plan_manager = VotePlanManager::new(vote_plan.clone(), HashSet::new());
        TestResult::from_bool(
            should_be_in_vote_time(&vote_plan, date) == vote_plan_manager.can_vote(date),
        )
    }

    #[quickcheck]
    pub fn vote_plan_manager_can_committee(vote_plan: VotePlan, date: BlockDate) -> TestResult {
        let vote_plan_manager = VotePlanManager::new(vote_plan.clone(), HashSet::new());
        TestResult::from_bool(
            should_be_in_committee_time(&vote_plan, date) == vote_plan_manager.can_committee(date),
        )
    }

    fn should_be_in_vote_time(vote_plan: &VotePlan, date: BlockDate) -> bool {
        let vote_start_date = vote_plan.vote_start();
        let vote_finish_date = vote_plan.vote_end();

        date >= vote_start_date && date < vote_finish_date
    }

    fn should_be_in_committee_time(vote_plan: &VotePlan, date: BlockDate) -> bool {
        let comittee_end_date = vote_plan.committee_end();
        let vote_finish_date = vote_plan.vote_end();

        date >= vote_finish_date && date < comittee_end_date
    }

    #[quickcheck]
    pub fn vote_plan_manager_plan_elapsed(vote_plan: VotePlan, date: BlockDate) -> TestResult {
        let vote_plan_manager = VotePlanManager::new(vote_plan.clone(), HashSet::new());
        let committee_end_date = vote_plan.committee_end();

        let vote_plan_elapsed = committee_end_date < date;
        TestResult::from_bool(vote_plan_elapsed == vote_plan_manager.vote_plan_elapsed(date))
    }

    #[test]
    pub fn vote_manager_vote_cast_different_id() {
        let vote_plan = VoteTestGen::vote_plan_with_proposals(1);
        let wrong_plan = VoteTestGen::vote_plan_with_proposals(1);
        let vote_plan_manager = VotePlanManager::new(vote_plan, HashSet::new());
        let vote_cast = VoteCast::new(wrong_plan.to_id(), 0, VoteTestGen::vote_cast_payload());

        assert_eq!(
            vote_plan_manager
                .vote(
                    BlockDate::first(),
                    TestGen::unspecified_account_identifier(),
                    vote_cast.clone()
                )
                .err()
                .unwrap(),
            VoteError::InvalidVotePlan {
                expected: vote_plan_manager.id().clone(),
                vote: vote_cast,
            }
        );
    }

    #[test]
    pub fn vote_manager_too_late_to_vote() {
        let vote_plan = VoteTestGen::vote_plan_with_proposals(1);
        let vote_plan_manager = VotePlanManager::new(vote_plan.clone(), HashSet::new());
        let vote_cast = VoteCast::new(vote_plan.to_id(), 0, VoteTestGen::vote_cast_payload());

        assert_eq!(
            vote_plan_manager
                .vote(
                    vote_plan.vote_end().next_epoch(),
                    TestGen::unspecified_account_identifier(),
                    vote_cast.clone()
                )
                .err()
                .unwrap(),
            VoteError::NotVoteTime {
                start: vote_plan.vote_start(),
                end: vote_plan.vote_end(),
                vote: vote_cast,
            }
        );
    }

    #[test]
    pub fn vote_manager_too_early_to_vote() {
        let vote_plan = VotePlan::new(
            BlockDate::from_epoch_slot_id(1, 0),
            BlockDate::from_epoch_slot_id(2, 0),
            BlockDate::from_epoch_slot_id(3, 0),
            VoteTestGen::proposals(3),
            vote::PayloadType::Public,
        );

        let vote_plan_manager = VotePlanManager::new(vote_plan.clone(), HashSet::new());
        let vote_cast = VoteCast::new(vote_plan.to_id(), 0, VoteTestGen::vote_cast_payload());

        assert_eq!(
            vote_plan_manager
                .vote(
                    BlockDate::first(),
                    TestGen::unspecified_account_identifier(),
                    vote_cast.clone()
                )
                .err()
                .unwrap(),
            VoteError::NotVoteTime {
                start: vote_plan.vote_start(),
                end: vote_plan.vote_end(),
                vote: vote_cast,
            }
        );
    }

    #[test]
    pub fn vote_manager_correct_vote() {
        let vote_plan = VotePlan::new(
            BlockDate::from_epoch_slot_id(1, 0),
            BlockDate::from_epoch_slot_id(2, 0),
            BlockDate::from_epoch_slot_id(3, 0),
            VoteTestGen::proposals(3),
            vote::PayloadType::Public,
        );

        let vote_plan_manager = VotePlanManager::new(vote_plan.clone(), HashSet::new());
        let vote_cast = VoteCast::new(vote_plan.to_id(), 0, VoteTestGen::vote_cast_payload());

        assert!(vote_plan_manager
            .vote(
                BlockDate::from_epoch_slot_id(1, 1),
                TestGen::unspecified_account_identifier(),
                vote_cast
            )
            .is_ok());
    }
}
