use crate::types::jorm_mock::{
    AccountId, AccountVote, Fragment, FragmentId, FragmentsProcessingSummary, ProposalIndex,
    Reason, RejectedInfo, VotePlanId, DEFAULT_POOL_NUMBER,
};
use chain_impl_mockchain::transaction::InputEnum;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct TimedType<T> {
    pub value: T,
    time: Instant,
}

impl<T: Hash> Hash for TimedType<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: PartialEq> PartialEq for TimedType<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T: Eq> Eq for TimedType<T> {}

impl<T> TimedType<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            time: Instant::now(),
        }
    }

    pub fn check_elapsed_time(&self, duration: Duration) -> bool {
        self.time.elapsed() >= duration
    }
}

type AccountVotes = HashMap<AccountId, HashMap<VotePlanId, HashSet<TimedType<ProposalIndex>>>>;

pub struct JormState {
    account_votes: AccountVotes,
    cleanup_timeout: Duration,
}

impl JormState {
    /// default `cleanup_timeout` value, 10 minutes
    pub const CLEANUP_TIMEOUT: Duration = Duration::from_secs(10 * 60);

    pub fn new(cleanup_timeout: Duration) -> Self {
        Self {
            account_votes: HashMap::new(),
            cleanup_timeout,
        }
    }

    fn add_account_vote(
        &mut self,
        account_id: AccountId,
        vote_plan_id: VotePlanId,
        proposal_index: ProposalIndex,
    ) -> bool {
        let vote_plan = self.account_votes.entry(account_id).or_default();
        let votes = vote_plan.entry(vote_plan_id).or_default();
        votes.insert(TimedType::new(proposal_index))
    }

    fn cleanup_votes(&mut self) {
        self.account_votes.retain(|_, vote_plans| {
            vote_plans.retain(|_, votes| {
                votes.retain(|vote| !vote.check_elapsed_time(self.cleanup_timeout));
                !votes.is_empty()
            });
            !vote_plans.is_empty()
        });
    }

    /// Accepts input fragments with a minimal validation for `VoteCast` fragment.
    ///
    /// - If it is a `VoteCast` fragment it is verified that the transaction contains 1 input and 1 witness,
    /// if it is valid storing account vote into the state.
    ///
    /// - If it is not a `VoteCast` fragment just returns it as accepted without storing and processing anything.
    pub fn accept_fragments(&mut self, fragments: Vec<Fragment>) -> FragmentsProcessingSummary {
        let mut accepted = vec![];
        let mut rejected = vec![];

        for fragment in fragments {
            let id = FragmentId(fragment.hash());

            match &fragment.0 {
                chain_impl_mockchain::fragment::Fragment::VoteCast(tx) => {
                    let tx = tx.as_slice();
                    // we've just verified that this is a valid transaction (i.e. contains 1 input and 1 witness)
                    match tx
                        .inputs()
                        .iter()
                        .map(|input| input.to_enum())
                        .zip(tx.witnesses().iter())
                        .next()
                    {
                        Some((InputEnum::AccountInput(account_id, _), _)) => {
                            match account_id.to_single_account() {
                                Some(account_id) => {
                                    let vote = tx.payload().into_payload();

                                    let account_id = AccountId(account_id.into());
                                    let vote_plan_id = VotePlanId(vote.vote_plan().clone().into());
                                    let proposal_index = ProposalIndex(vote.proposal_index());

                                    if self.add_account_vote(
                                        account_id,
                                        vote_plan_id,
                                        proposal_index,
                                    ) {
                                        accepted.push(id);
                                    } else {
                                        rejected.push(RejectedInfo {
                                            id,
                                            pool_number: DEFAULT_POOL_NUMBER,
                                            reason: Reason::FragmentAlreadyInLog,
                                        })
                                    }
                                }
                                None => rejected.push(RejectedInfo {
                                    id,
                                    pool_number: DEFAULT_POOL_NUMBER,
                                    reason: Reason::FragmentInvalid,
                                }),
                            }
                        }
                        _ => rejected.push(RejectedInfo {
                            id,
                            pool_number: DEFAULT_POOL_NUMBER,
                            reason: Reason::FragmentInvalid,
                        }),
                    }
                }
                _ => accepted.push(id),
            }
        }

        FragmentsProcessingSummary { accepted, rejected }
    }

    pub fn get_account_votes(&mut self, account_id: &AccountId) -> Vec<AccountVote> {
        self.cleanup_votes();

        match self.account_votes.get(account_id) {
            Some(vote_plans) => vote_plans
                .clone()
                .into_iter()
                .map(|(vote_plan_id, votes)| AccountVote {
                    vote_plan_id,
                    votes: votes.into_iter().map(|vote| vote.value).collect(),
                })
                .collect(),
            None => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_impl_mockchain::key::AccountPublicKey;
    use quickcheck_macros::quickcheck;
    use std::str::FromStr;

    #[test]
    fn timed_type_test() {
        let val = TimedType::new(1);

        assert_eq!(val.value, 1);

        let duration = Duration::from_secs(5);
        assert!(!val.check_elapsed_time(duration));
        std::thread::sleep(duration);
        assert!(val.check_elapsed_time(duration));
    }

    #[quickcheck]
    fn accept_fragments_test(
        f1: chain_impl_mockchain::fragment::Fragment,
        f2: chain_impl_mockchain::fragment::Fragment,
        f3: chain_impl_mockchain::fragment::Fragment,
    ) {
        let fragments = vec![f1, f2, f3];

        let mut state = JormState::new(JormState::CLEANUP_TIMEOUT);

        let res = state.accept_fragments(
            fragments
                .clone()
                .into_iter()
                .map(Fragment)
                .collect::<Vec<_>>(),
        );

        assert_eq!(res.accepted.len() + res.rejected.len(), fragments.len());
    }

    #[test]
    fn account_votes_test() {
        let mut state = JormState::new(JormState::CLEANUP_TIMEOUT);

        let account_id = AccountId(
            AccountPublicKey::from_str("0000000000000000000000000000000000000000").unwrap(),
        );
        let vote_plan_id = VotePlanId(
            jormungandr_lib::interfaces::VotePlanId::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
        );
        let proposal_index = ProposalIndex(1);

        assert!(state.add_account_vote(
            account_id.clone(),
            vote_plan_id.clone(),
            proposal_index.clone()
        ));
        assert!(!state.add_account_vote(
            account_id.clone(),
            vote_plan_id.clone(),
            proposal_index.clone()
        ));

        assert_eq!(
            state.get_account_votes(&account_id),
            vec![AccountVote {
                vote_plan_id,
                votes: vec![proposal_index],
            }]
        );
        assert_eq!(
            state.get_account_votes(&AccountId(
                AccountPublicKey::from_str("0000000000000000000000000000000000000001").unwrap(),
            )),
            vec![]
        );
    }

    #[test]
    fn cleanup_votes_test() {
        let duration = Duration::from_secs(5);
        let mut state = JormState::new(duration);

        let account_id = AccountId(
            AccountPublicKey::from_str("0000000000000000000000000000000000000000").unwrap(),
        );
        let vote_plan_id = VotePlanId(
            jormungandr_lib::interfaces::VotePlanId::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
        );
        let proposal_index = ProposalIndex(1);
        assert!(state.add_account_vote(
            account_id.clone(),
            vote_plan_id.clone(),
            proposal_index.clone()
        ));

        assert!(!state.get_account_votes(&account_id).is_empty());

        std::thread::sleep(duration);
        assert!(state.get_account_votes(&account_id).is_empty());
    }
}
