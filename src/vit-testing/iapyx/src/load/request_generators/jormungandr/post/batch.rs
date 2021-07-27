use crate::load::{MultiController, MultiControllerError};
use crate::Proposal;
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
}

impl BatchWalletRequestGen {
    pub fn new(
        multi_controller: MultiController,
        batch_size: usize,
        use_v1: bool,
        update_account_before_vote: bool,
    ) -> Self {
        let proposals = multi_controller.proposals().unwrap();
        let options = proposals[0]
            .chain_vote_options
            .0
            .values()
            .cloned()
            .collect();
        Self {
            batch_size,
            use_v1,
            multi_controller,
            rand: OsRng,
            proposals,
            options,
            wallet_index: 0,
            update_account_before_vote,
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

        if self.update_account_before_vote {
            self.multi_controller.update_wallet_state(wallet_index);
        }

        let batch_size = self.batch_size;
        let proposals = self.proposals.clone();
        let options = self.options.clone();

        let proposals: Vec<Proposal> =
            std::iter::from_fn(|| Some(self.next_usize() % self.proposals.len()))
                .take(batch_size)
                .map(|index| proposals.get(index).unwrap().clone())
                .collect();

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
