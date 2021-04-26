use crate::load::{MultiController, MultiControllerError};
use crate::Proposal;
use jortestkit::load::{Id, RequestFailure, RequestGenerator};
use rand::RngCore;
use rand_core::OsRng;
use wallet_core::Choice;

pub struct BatchWalletRequestGen {
    rand: OsRng,
    batch_size: usize,
    multi_controller: MultiController,
    proposals: Vec<Proposal>,
    options: Vec<u8>,
}

impl BatchWalletRequestGen {
    pub fn new(multi_controller: MultiController, batch_size: usize) -> Self {
        let proposals = multi_controller.proposals().unwrap();
        let options = proposals[0]
            .chain_vote_options
            .0
            .values()
            .cloned()
            .collect();
        Self {
            batch_size,
            multi_controller,
            rand: OsRng,
            proposals,
            options,
        }
    }

    pub fn next_usize(&mut self) -> usize {
        self.rand.next_u32() as usize
    }

    pub fn random_votes(&mut self) -> Result<Vec<Option<Id>>, MultiControllerError> {
        let wallet_index = self.next_usize() % self.multi_controller.wallet_count();
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
                .map(|index| Choice::new(*options.get(index).clone().unwrap()))
                .collect();

        self.multi_controller.refresh_wallet(wallet_index)?;

        Ok(proposals
            .iter()
            .zip(choices)
            .map(|(proposal, choice)| {
                Some(
                    self.multi_controller
                        .vote(wallet_index, proposal, choice)
                        .unwrap()
                        .to_string(),
                )
            })
            .collect())
    }
}

impl RequestGenerator for BatchWalletRequestGen {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        Ok(self
            .random_votes()
            .map_err(|e| RequestFailure::General(format!("{:?}", e)))?)
    }
}
