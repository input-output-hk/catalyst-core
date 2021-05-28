use crate::load::{MultiController, MultiControllerError};
use crate::Proposal;
use chain_impl_mockchain::fragment::FragmentId;
use jortestkit::load::{Id, RequestFailure, RequestGenerator};
use rand::{seq::SliceRandom, Rng};
use rand_core::OsRng;
use wallet_core::Choice;

pub struct WalletRequestGen {
    rand: OsRng,
    multi_controller: MultiController,
    proposals: Vec<Proposal>,
    options: Vec<u8>,
}

impl WalletRequestGen {
    pub fn new(multi_controller: MultiController) -> Self {
        let proposals = multi_controller.proposals().unwrap();
        let options = proposals[0]
            .chain_vote_options
            .0
            .values()
            .cloned()
            .collect();

        Self {
            multi_controller,
            rand: OsRng,
            proposals,
            options,
        }
    }

    pub fn random_vote(&mut self) -> Result<FragmentId, MultiControllerError> {
        let wallet_index = self.rand.gen_range(0..self.multi_controller.wallet_count());

        let proposal = self.proposals.choose(&mut self.rand).unwrap();
        let choice = Choice::new(*self.options.choose(&mut self.rand).unwrap());

        self.multi_controller.refresh_wallet(wallet_index)?;
        self.multi_controller.vote(wallet_index, &proposal, choice)
    }
}

impl RequestGenerator for WalletRequestGen {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        let id = self
            .random_vote()
            .map_err(|e| RequestFailure::General(format!("{:?}", e)))?;
        Ok(vec![Some(id.to_string())])
    }
}
