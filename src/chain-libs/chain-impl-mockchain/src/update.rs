//use crate::certificate::{verify_certificate, HasPublicKeys, SignatureRaw};
use crate::certificate::{UpdateProposal, UpdateProposalId, UpdateVote, UpdateVoterId};
use crate::date::BlockDate;
use crate::setting::{ActiveSlotsCoeffError, Settings};
use imhamt::Hamt;
use std::collections::hash_map::DefaultHasher;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateState {
    pub proposals: Hamt<DefaultHasher, UpdateProposalId, UpdateProposalState>,
}

impl UpdateState {
    pub fn new() -> Self {
        UpdateState {
            proposals: Hamt::new(),
        }
    }

    pub fn apply_proposal(
        mut self,
        proposal_id: UpdateProposalId,
        proposal: UpdateProposal,
        settings: &Settings,
        cur_date: BlockDate,
    ) -> Result<Self, Error> {
        let proposer_id = proposal.proposer_id();

        if !settings.bft_leaders.contains(proposer_id) {
            return Err(Error::BadProposer(proposal_id, proposer_id.clone()));
        }

        self.proposals = self
            .proposals
            .insert(
                proposal_id,
                UpdateProposalState {
                    proposal: proposal.clone(),
                    proposal_date: cur_date,
                    votes: Hamt::new(),
                },
            )
            .map_err(|_| Error::DuplicateProposal(proposal_id))?;
        Ok(self)
    }

    pub fn apply_vote(mut self, vote: &UpdateVote, settings: &Settings) -> Result<Self, Error> {
        if !settings.bft_leaders.contains(vote.voter_id()) {
            return Err(Error::BadVoter(
                *vote.proposal_id(),
                vote.voter_id().clone(),
            ));
        }

        let new = self.proposals.update(vote.proposal_id(), |proposal| {
            let mut proposal_new = proposal.clone();
            proposal_new.votes = proposal.votes.insert(vote.voter_id().clone(), ())?;
            Ok::<_, imhamt::InsertError>(Some(proposal_new))
        });

        match new {
            Err(imhamt::UpdateError::KeyNotFound) => {
                Err(Error::VoteForMissingProposal(*vote.proposal_id()))
            }
            Err(imhamt::UpdateError::ValueCallbackError(_)) => Err(Error::DuplicateVote(
                *vote.proposal_id(),
                vote.voter_id().clone(),
            )),
            Ok(new) => {
                self.proposals = new;
                Ok(self)
            }
        }
    }

    pub fn process_proposals(
        mut self,
        mut settings: Settings,
        prev_date: BlockDate,
        new_date: BlockDate,
    ) -> Result<(Self, Settings), Error> {
        let mut expired_ids = vec![];

        assert!(prev_date < new_date);

        let mut proposals: Vec<(&UpdateProposalId, &UpdateProposalState)> =
            self.proposals.iter().collect();

        // sort proposals by the date
        proposals.sort_by(|(a_id, a_state), (b_id, b_state)| {
            match a_state.proposal_date.cmp(&b_state.proposal_date) {
                std::cmp::Ordering::Equal => a_id.cmp(b_id),
                res => res,
            }
        });

        // If we entered a new epoch, then delete expired update
        // proposals and apply accepted update proposals.
        if prev_date.epoch < new_date.epoch {
            for (proposal_id, proposal_state) in proposals {
                // If a majority of BFT leaders voted for the
                // proposal, then apply it.
                if proposal_state.votes.size() > settings.bft_leaders.len() / 2 {
                    settings = settings.apply(proposal_state.proposal.changes())?;
                    expired_ids.push(*proposal_id);
                } else if proposal_state.proposal_date.epoch + settings.proposal_expiration
                    < new_date.epoch
                {
                    expired_ids.push(*proposal_id);
                }
            }

            for proposal_id in expired_ids {
                self.proposals = self
                    .proposals
                    .remove(&proposal_id)
                    .expect("proposal does not exist");
            }
        }

        Ok((self, settings))
    }
}

impl Default for UpdateState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateProposalState {
    pub proposal: UpdateProposal,
    pub proposal_date: BlockDate,
    pub votes: Hamt<DefaultHasher, UpdateVoterId, ()>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /*
    InvalidCurrentBlockId(Hash, Hash),
    UpdateIsInvalid,
     */
    BadProposalSignature(UpdateProposalId, UpdateVoterId),
    BadProposer(UpdateProposalId, UpdateVoterId),
    DuplicateProposal(UpdateProposalId),
    VoteForMissingProposal(UpdateProposalId),
    BadVoteSignature(UpdateProposalId, UpdateVoterId),
    BadVoter(UpdateProposalId, UpdateVoterId),
    DuplicateVote(UpdateProposalId, UpdateVoterId),
    ReadOnlySetting,
    BadBftSlotsRatio(crate::milli::Milli),
    BadConsensusGenesisPraosActiveSlotsCoeff(ActiveSlotsCoeffError),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            /*
            Error::InvalidCurrentBlockId(current_one, update_one) => {
                write!(f, "Cannot apply Setting Update. Update needs to be applied to from block {:?} but received {:?}", update_one, current_one)
            }
            Error::UpdateIsInvalid => write!(
                f,
                "Update does not apply to current state"
            ),
             */
            Error::BadProposalSignature(proposal_id, proposer_id) => write!(
                f,
                "Proposal {} from {:?} has an incorrect signature",
                proposal_id, proposer_id
            ),
            Error::BadProposer(proposal_id, proposer_id) => write!(
                f,
                "Proposer {:?} for proposal {} is not a BFT leader",
                proposer_id, proposal_id
            ),
            Error::DuplicateProposal(proposal_id) => {
                write!(f, "Received a duplicate proposal {}", proposal_id)
            }
            Error::VoteForMissingProposal(proposal_id) => write!(
                f,
                "Received a vote for a non-existent proposal {}",
                proposal_id
            ),
            Error::BadVoteSignature(proposal_id, voter_id) => write!(
                f,
                "Vote from {:?} for proposal {} has an incorrect signature",
                voter_id, proposal_id
            ),
            Error::BadVoter(proposal_id, voter_id) => write!(
                f,
                "Voter {:?} for proposal {} is not a BFT leader",
                voter_id, proposal_id
            ),
            Error::DuplicateVote(proposal_id, voter_id) => write!(
                f,
                "Received a duplicate vote from {:?} for proposal {}",
                voter_id, proposal_id
            ),
            Error::ReadOnlySetting => write!(
                f,
                "Received a proposal to modify a chain parameter that can only be set in block 0"
            ),
            Error::BadBftSlotsRatio(m) => {
                write!(f, "Cannot set BFT slots ratio to invalid value {}", m)
            }
            Error::BadConsensusGenesisPraosActiveSlotsCoeff(err) => write!(
                f,
                "Cannot set consensus genesis praos active slots coefficient: {}",
                err
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<ActiveSlotsCoeffError> for Error {
    fn from(err: ActiveSlotsCoeffError) -> Self {
        Error::BadConsensusGenesisPraosActiveSlotsCoeff(err)
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    use crate::certificate::UpdateProposal;
    #[cfg(test)]
    use crate::testing::serialization::serialization_bijection;
    #[cfg(test)]
    use crate::{
        config::ConfigParam,
        fragment::config::ConfigParams,
        testing::{data::LeaderPair, TestGen},
    };
    #[cfg(test)]
    use chain_addr::Discrimination;
    #[cfg(test)]
    use quickcheck::TestResult;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;
    use std::iter;

    impl Arbitrary for UpdateProposalState {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let size = usize::arbitrary(g);
            Self {
                proposal: UpdateProposal::arbitrary(g),
                proposal_date: BlockDate::arbitrary(g),
                votes: iter::from_fn(|| Some((UpdateVoterId::arbitrary(g), ())))
                    .take(size)
                    .collect(),
            }
        }
    }

    #[cfg(test)]
    fn apply_update_proposal(
        update_state: UpdateState,
        proposal_id: UpdateProposalId,
        config_param: ConfigParam,
        proposer: &LeaderPair,
        settings: &Settings,
        block_date: BlockDate,
    ) -> Result<UpdateState, Error> {
        let update_proposal = UpdateProposal::new(ConfigParams(vec![config_param]), proposer.id());
        update_state.apply_proposal(proposal_id, update_proposal, &settings, block_date)
    }

    #[cfg(test)]
    fn apply_update_vote(
        update_state: UpdateState,
        proposal_id: UpdateProposalId,
        proposer: &LeaderPair,
        settings: &Settings,
    ) -> Result<UpdateState, Error> {
        let signed_update_vote = UpdateVote::new(proposal_id, proposer.id());
        update_state.apply_vote(&signed_update_vote, &settings)
    }

    quickcheck! {
        fn update_proposal_serialize_deserialize_bijection(update_proposal: UpdateProposal) -> TestResult {
            serialization_bijection(update_proposal)
        }
    }

    #[test]
    fn apply_proposal_with_unknown_proposer_should_return_error() {
        // data
        let unknown_leader = TestGen::leader_pair();
        let block_date = BlockDate::first();
        let proposal_id = TestGen::hash();
        let config_param = ConfigParam::SlotsPerEpoch(100);
        //setup
        let update_state = UpdateState::new();
        let settings = Settings::new();

        assert_eq!(
            apply_update_proposal(
                update_state,
                proposal_id,
                config_param,
                &unknown_leader,
                &settings,
                block_date,
            ),
            Err(Error::BadProposer(proposal_id, unknown_leader.id()))
        );
    }

    #[test]
    fn apply_duplicated_proposal_should_return_error() {
        // data
        let proposal_id = TestGen::hash();
        let block_date = BlockDate::first();
        let config_param = ConfigParam::SlotsPerEpoch(100);
        //setup
        let mut update_state = UpdateState::new();

        let leaders = TestGen::leaders_pairs()
            .take(5)
            .collect::<Vec<LeaderPair>>();
        let proposer = &leaders[0];
        let settings = TestGen::settings(leaders.clone());

        update_state = apply_update_proposal(
            update_state,
            proposal_id,
            config_param.clone(),
            proposer,
            &settings,
            block_date,
        )
        .expect("failed while applying first proposal");

        assert_eq!(
            apply_update_proposal(
                update_state,
                proposal_id,
                config_param,
                proposer,
                &settings,
                block_date
            ),
            Err(Error::DuplicateProposal(proposal_id))
        );
    }

    #[test]
    fn test_add_vote_for_non_existing_proposal_should_return_error() {
        let mut update_state = UpdateState::new();
        let proposal_id = TestGen::hash();
        let unknown_proposal_id = TestGen::hash();
        let block_date = BlockDate::first();
        let config_param = ConfigParam::SlotsPerEpoch(100);
        let leaders = TestGen::leaders_pairs()
            .take(5)
            .collect::<Vec<LeaderPair>>();
        let proposer = &leaders[0];
        let settings = TestGen::settings(leaders.clone());

        // Apply proposal
        update_state = apply_update_proposal(
            update_state,
            proposal_id,
            config_param,
            proposer,
            &settings,
            block_date,
        )
        .expect("failed while applying first proposal");

        // Apply vote for unknown proposal
        assert_eq!(
            apply_update_vote(update_state, unknown_proposal_id, proposer, &settings),
            Err(Error::VoteForMissingProposal(unknown_proposal_id))
        );
    }

    #[test]
    fn test_add_duplicated_vote_should_return_error() {
        let mut update_state = UpdateState::new();
        let proposal_id = TestGen::hash();
        let block_date = BlockDate::first();
        let config_param = ConfigParam::SlotsPerEpoch(100);

        let leaders = TestGen::leaders_pairs()
            .take(5)
            .collect::<Vec<LeaderPair>>();
        let proposer = &leaders[0];
        let settings = TestGen::settings(leaders.clone());

        update_state = apply_update_proposal(
            update_state,
            proposal_id,
            config_param,
            proposer,
            &settings,
            block_date,
        )
        .expect("failed while applying proposal");

        // Apply vote
        update_state = apply_update_vote(update_state, proposal_id, proposer, &settings)
            .expect("failed while applying first vote");

        // Apply duplicated vote
        assert_eq!(
            apply_update_vote(update_state, proposal_id, proposer, &settings),
            Err(Error::DuplicateVote(proposal_id, proposer.id()))
        );
    }

    #[test]
    fn test_add_vote_from_unknown_voter_should_return_error() {
        let mut update_state = UpdateState::new();
        let proposal_id = TestGen::hash();
        let unknown_leader = TestGen::leader_pair();
        let block_date = BlockDate::first();
        let config_param = ConfigParam::SlotsPerEpoch(100);

        let leaders = TestGen::leaders_pairs()
            .take(5)
            .collect::<Vec<LeaderPair>>();
        let proposer = &leaders[0];
        let settings = TestGen::settings(leaders.clone());

        update_state = apply_update_proposal(
            update_state,
            proposal_id,
            config_param,
            proposer,
            &settings,
            block_date,
        )
        .expect("failed while applying proposal");

        // Apply vote for unknown leader
        assert_eq!(
            apply_update_vote(update_state, proposal_id, &unknown_leader, &settings),
            Err(Error::BadVoter(proposal_id, unknown_leader.id()))
        );
    }

    #[test]
    fn process_proposals_for_readonly_setting_should_return_error() {
        let mut update_state = UpdateState::new();
        let proposal_id = TestGen::hash();
        let proposer = TestGen::leader_pair();
        let block_date = BlockDate::first();
        let readonly_setting = ConfigParam::Discrimination(Discrimination::Test);

        let settings = TestGen::settings(vec![proposer.clone()]);

        update_state = apply_update_proposal(
            update_state,
            proposal_id,
            readonly_setting,
            &proposer,
            &settings,
            block_date,
        )
        .expect("failed while applying proposal");

        // Apply vote
        update_state = apply_update_vote(update_state, proposal_id, &proposer, &settings)
            .expect("failed while applying vote");

        assert_eq!(
            update_state.process_proposals(settings, block_date, block_date.next_epoch()),
            Err(Error::ReadOnlySetting)
        );
    }

    #[test]
    pub fn process_proposal_is_by_id_ordered() {
        let mut update_state = UpdateState::new();
        let first_proposal_id = TestGen::hash();
        let second_proposal_id = TestGen::hash();
        let first_proposer = TestGen::leader_pair();
        let second_proposer = TestGen::leader_pair();
        let block_date = BlockDate::first();
        let first_update = ConfigParam::SlotsPerEpoch(100);
        let second_update = ConfigParam::SlotsPerEpoch(200);

        let settings = TestGen::settings(vec![first_proposer.clone(), second_proposer.clone()]);

        // Apply proposal
        update_state = apply_update_proposal(
            update_state,
            first_proposal_id,
            first_update,
            &first_proposer,
            &settings,
            block_date,
        )
        .expect("failed while applying proposal");

        // Apply vote
        update_state =
            apply_update_vote(update_state, first_proposal_id, &first_proposer, &settings)
                .expect("failed while applying vote");

        // Apply vote
        update_state =
            apply_update_vote(update_state, first_proposal_id, &second_proposer, &settings)
                .expect("failed while applying vote");

        // Apply proposal
        update_state = apply_update_proposal(
            update_state,
            second_proposal_id,
            second_update,
            &second_proposer,
            &settings,
            block_date,
        )
        .expect("failed while applying proposal");

        // Apply vote
        update_state =
            apply_update_vote(update_state, second_proposal_id, &first_proposer, &settings)
                .expect("failed while applying vote");

        // Apply vote
        update_state = apply_update_vote(
            update_state,
            second_proposal_id,
            &second_proposer,
            &settings,
        )
        .expect("failed while applying vote");

        let (update_state, settings) = update_state
            .process_proposals(settings, block_date, block_date.next_epoch())
            .expect("error while processing proposal");

        match first_proposal_id.cmp(&second_proposal_id) {
            std::cmp::Ordering::Less => assert_eq!(settings.slots_per_epoch, 200),
            std::cmp::Ordering::Greater => assert_eq!(settings.slots_per_epoch, 100),
            _ => {}
        }

        assert_eq!(update_state.proposals.size(), 0);
    }

    #[test]
    pub fn process_proposal_is_by_time_ordered() {
        let mut update_state = UpdateState::new();
        let first_proposal_id = TestGen::hash();
        let second_proposal_id = TestGen::hash();
        let first_proposer = TestGen::leader_pair();
        let second_proposer = TestGen::leader_pair();
        let first_proposal_block_date = BlockDate::first();
        let second_proposal_block_date = BlockDate {
            slot_id: first_proposal_block_date.slot_id + 1,
            ..first_proposal_block_date
        };
        let first_update = ConfigParam::SlotsPerEpoch(100);
        let second_update = ConfigParam::SlotsPerEpoch(200);

        let settings = TestGen::settings(vec![first_proposer.clone(), second_proposer.clone()]);

        // Apply proposal
        update_state = apply_update_proposal(
            update_state,
            first_proposal_id,
            first_update,
            &first_proposer,
            &settings,
            first_proposal_block_date,
        )
        .expect("failed while applying proposal");

        // Apply vote
        update_state =
            apply_update_vote(update_state, first_proposal_id, &first_proposer, &settings)
                .expect("failed while applying vote");

        // Apply vote
        update_state =
            apply_update_vote(update_state, first_proposal_id, &second_proposer, &settings)
                .expect("failed while applying vote");

        // Apply proposal
        update_state = apply_update_proposal(
            update_state,
            second_proposal_id,
            second_update,
            &second_proposer,
            &settings,
            second_proposal_block_date,
        )
        .expect("failed while applying proposal");

        // Apply vote
        update_state =
            apply_update_vote(update_state, second_proposal_id, &first_proposer, &settings)
                .expect("failed while applying vote");

        // Apply vote
        update_state = apply_update_vote(
            update_state,
            second_proposal_id,
            &second_proposer,
            &settings,
        )
        .expect("failed while applying vote");

        let (update_state, settings) = update_state
            .process_proposals(
                settings,
                first_proposal_block_date,
                first_proposal_block_date.next_epoch(),
            )
            .expect("error while processing proposal");

        assert_eq!(settings.slots_per_epoch, 200);

        assert_eq!(update_state.proposals.size(), 0);
    }

    #[cfg(test)]
    #[derive(Debug, Copy, Clone)]
    struct ExpiryBlockDate {
        pub block_date: BlockDate,
        pub proposal_expiration: u32,
    }

    #[cfg(test)]
    impl ExpiryBlockDate {
        pub fn block_date(&self) -> BlockDate {
            self.block_date
        }

        pub fn proposal_expiration(&self) -> u32 {
            self.proposal_expiration
        }

        pub fn get_last_epoch(&self) -> u32 {
            self.block_date().epoch + self.proposal_expiration() + 1
        }
    }

    #[cfg(test)]
    impl Arbitrary for ExpiryBlockDate {
        fn arbitrary<G: Gen>(gen: &mut G) -> Self {
            let mut block_date = BlockDate::arbitrary(gen);
            block_date.epoch %= 10;
            let proposal_expiration = u32::arbitrary(gen) % 10;
            ExpiryBlockDate {
                block_date,
                proposal_expiration,
            }
        }
    }

    #[quickcheck]
    fn rejected_proposals_are_removed_after_expiration_period(
        expiry_block_data: ExpiryBlockDate,
    ) -> TestResult {
        let proposal_date = expiry_block_data.block_date();
        let proposal_expiration = expiry_block_data.proposal_expiration();

        let mut update_state = UpdateState::new();
        let proposal_id = TestGen::hash();
        let proposer = TestGen::leader_pair();
        let update = ConfigParam::SlotsPerEpoch(100);

        let mut settings = TestGen::settings(vec![proposer.clone()]);
        settings.proposal_expiration = proposal_expiration;

        // Apply proposal
        update_state = apply_update_proposal(
            update_state,
            proposal_id,
            update,
            &proposer,
            &settings,
            proposal_date,
        )
        .expect("failed while applying proposal");

        let mut current_block_date = BlockDate::first();

        // Traverse through epoch and check if proposal is still in queue
        // if proposal expiration period is not exceeded after that
        // proposal should be removed from proposal collection
        for _i in 0..expiry_block_data.get_last_epoch() {
            let (update_state, _settings) = update_state
                .clone()
                .process_proposals(
                    settings.clone(),
                    current_block_date,
                    current_block_date.next_epoch(),
                )
                .expect("error while processing proposal");

            if proposal_date.epoch + proposal_expiration <= current_block_date.epoch {
                assert_eq!(update_state.proposals.size(), 0);
            } else {
                assert_eq!(update_state.proposals.size(), 1);
            }
            current_block_date = current_block_date.next_epoch()
        }

        TestResult::passed()
    }
}
