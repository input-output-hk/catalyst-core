use crate::types::jorm_mock::{
    AccountId, AccountVote, Fragment, FragmentId, FragmentsProcessingSummary, ProposalIndex,
    Reason, RejectedInfo, VotePlanId, DEFAULT_POOL_NUMBER,
};
use chain_impl_mockchain::transaction::{InputEnum, Witness};
use std::collections::HashMap;

#[derive(Default)]
pub struct JormState {
    account_votes: HashMap<AccountId, HashMap<VotePlanId, Vec<ProposalIndex>>>,
}

impl JormState {
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
                        .unwrap()
                    {
                        (InputEnum::AccountInput(account_id, _), Witness::Account(_, _)) => {
                            match account_id.to_single_account() {
                                Some(account_id) => {
                                    let vote = tx.payload().into_payload();

                                    let account_id = AccountId(account_id.into());
                                    let vote_plan_id = VotePlanId(vote.vote_plan().clone().into());
                                    let proposal_index = ProposalIndex(vote.proposal_index());

                                    let vote_plan =
                                        self.account_votes.entry(account_id).or_default();
                                    let votes = vote_plan.entry(vote_plan_id).or_default();
                                    votes.push(proposal_index);

                                    accepted.push(id);
                                }
                                None => rejected.push(RejectedInfo {
                                    id,
                                    pool_number: DEFAULT_POOL_NUMBER,
                                    reason: Reason::FragmentInvalid,
                                }),
                            }
                        }
                        (_, _) => rejected.push(RejectedInfo {
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

    pub fn get_account_votes(&self, account_id: &AccountId) -> Vec<AccountVote> {
        match self.account_votes.get(account_id) {
            Some(vote_plans) => vote_plans
                .clone()
                .into_iter()
                .map(|(vote_plan_id, votes)| AccountVote {
                    vote_plan_id: vote_plan_id,
                    votes: votes,
                })
                .collect(),
            None => vec![],
        }
    }
}
