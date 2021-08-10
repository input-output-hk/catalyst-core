use crate::load::{MultiController, MultiControllerError};
use crate::Proposal;
use crate::Wallet;
use jormungandr_testing_utils::testing::VoteCastCounter;
use jortestkit::load::{Id, Request, RequestFailure, RequestGenerator};
use rand::RngCore;
use rand_core::OsRng;
use std::time::Instant;
use wallet_core::Choice;

pub struct BatchWalletRequestGen {
    rand: OsRng,
    batch_size: usize,
    multi_controller: MultiController,
    proposals: Vec<Proposal>,
    options: Vec<u8>,
    use_v1: bool,
    wallet_index: usize,
    update_account_before_vote: bool,
    vote_cast_counter: VoteCastCounter,
}

impl BatchWalletRequestGen {
    pub fn new(
        multi_controller: MultiController,
        batch_size: usize,
        use_v1: bool,
        update_account_before_vote: bool,
    ) -> Self {
        let proposals = multi_controller.proposals().unwrap();
        let vote_plans = multi_controller.backend().vote_plan_statuses().unwrap();
        let options = proposals[0]
            .chain_vote_options
            .0
            .values()
            .cloned()
            .collect();

        let vote_cast_counter = VoteCastCounter::new(
            multi_controller.wallet_count(),
            vote_plans
                .iter()
                .map(|v| (v.id.into(), v.proposals.len() as u8))
                .collect(),
        );

        Self {
            batch_size,
            use_v1,
            multi_controller,
            rand: OsRng,
            proposals,
            options,
            wallet_index: 0,
            update_account_before_vote,
            vote_cast_counter,
        }
    }

    pub fn next_usize(&mut self) -> usize {
        self.rand.next_u32() as usize
    }

    pub fn random_votes(&mut self) -> Result<Vec<Option<Id>>, MultiControllerError> {
        let wallet_index = {
            self.wallet_index += 1;
            if self.wallet_index >= self.multi_controller.wallet_count() {
                self.wallet_index = 0;
            }
            self.wallet_index
        };

        // update state of wallet only before first vote.
        // Then relay on mechanism of spending counter auto-update
        if self.update_account_before_vote {
            self.multi_controller
                .update_wallet_state_if(wallet_index, &|wallet: &Wallet| {
                    wallet.spending_counter() == 0
                });
        }

        let batch_size = self.batch_size;
        let options = self.options.clone();

        let counter = self
            .vote_cast_counter
            .advance_batch(batch_size, wallet_index)
            .unwrap();

        let mut proposals = Vec::new();

        counter.iter().for_each(|item| {
            for i in item.range() {
                proposals.push(
                    self.proposals
                        .iter()
                        .find(|x| {
                            x.chain_voteplan_id == item.id().to_string()
                                && (x.internal_id % u8::MAX as i64) == i as i64
                        })
                        .unwrap()
                        .clone(),
                );
            }
        });

        let choices: Vec<Choice> =
            std::iter::from_fn(|| Some(self.next_usize() % self.options.len()))
                .take(batch_size)
                .map(|index| Choice::new(*options.get(index).unwrap()))
                .collect();

        self.multi_controller
            .votes_batch(
                wallet_index,
                self.use_v1,
                proposals.iter().zip(choices).collect(),
            )
            .map(|x| {
                x.into_iter()
                    .map(|s| Some(s.to_string()))
                    .collect::<Vec<Option<Id>>>()
            })
    }
}

impl RequestGenerator for BatchWalletRequestGen {
    fn split(mut self) -> (Self, Option<Self>) {
        let wallets_len = self.multi_controller.wallets.len();
        if wallets_len <= 1 {
            return (self, None);
        }
        let wallets = self.multi_controller.wallets.split_off(wallets_len / 2);
        let new_gen = Self {
            rand: self.rand,
            multi_controller: MultiController {
                wallets,
                backend: self.multi_controller.backend.clone(),
                settings: self.multi_controller.settings.clone(),
            },
            proposals: self.proposals.clone(),
            options: self.options.clone(),
            use_v1: self.use_v1,
            batch_size: self.batch_size,
            wallet_index: 0,
            update_account_before_vote: self.update_account_before_vote,
            vote_cast_counter: self.vote_cast_counter.clone(),
        };

        (self, Some(new_gen))
    }

    fn next(&mut self) -> Result<Request, RequestFailure> {
        let start = Instant::now();
        match self.random_votes() {
            Ok(ids) => Ok(Request {
                ids,
                duration: start.elapsed(),
            }),
            Err(e) => Err(RequestFailure::General(format!("{:?}", e))),
        }
    }
}
