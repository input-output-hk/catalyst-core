use crate::load::{MultiController, MultiControllerError};
use crate::Proposal;
use chain_impl_mockchain::fragment::FragmentId;
use jortestkit::load::{Id, RequestFailure, RequestGenerator};
use rand::RngCore;
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

    pub fn next_usize(&mut self) -> usize {
        self.rand.next_u32() as usize
    }

    pub fn random_vote(&mut self) -> Result<FragmentId, MultiControllerError> {
        let proposal_index = self.next_usize() % self.proposals.len();
        let wallet_index = self.next_usize() % self.multi_controller.wallet_count();

        let proposal: Proposal = self.proposals.get(proposal_index).unwrap().clone();

        let choice_index = self.next_usize() % self.options.len();
        let choice = Choice::new(*self.options.get(choice_index).unwrap());

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
