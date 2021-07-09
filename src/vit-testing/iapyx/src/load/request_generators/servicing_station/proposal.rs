use crate::backend::VitStationRestClient;
use crate::rand::RngCore;
use crate::Proposal;
use jortestkit::load::{Id, RequestFailure, RequestGenerator};
use rand::rngs::OsRng;

pub struct ProposalsRequestGen {
    client: VitStationRestClient,
}

impl ProposalsRequestGen {
    pub fn new(client: VitStationRestClient) -> Self {
        Self { client }
    }
}

impl RequestGenerator for ProposalsRequestGen {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        self.client
            .proposals_raw()
            .map(|response| vec![])
            .map_err(|e| RequestFailure::General(format!("{:?}", e)))
    }
}

pub struct ProposalRequestGen {
    client: VitStationRestClient,
    proposals: Vec<Proposal>,
    rand: OsRng,
}

impl ProposalRequestGen {
    pub fn new(client: VitStationRestClient, proposals: Vec<Proposal>) -> Self {
        Self {
            client,
            proposals,
            rand: OsRng,
        }
    }

    fn next_usize(&mut self) -> usize {
        self.rand.next_u32() as usize
    }

    pub fn random_proposal(&mut self) -> Result<(), RequestFailure> {
        let index = self.next_usize() % self.proposals.len();
        let proposal = self.proposals.get(index).unwrap().clone();
        self.client
            .proposal_raw(&proposal.proposal_id)
            .map_err(|e| RequestFailure::General(format!("{:?}", e)))?;
        Ok(())
    }
}

impl RequestGenerator for ProposalRequestGen {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        self.random_proposal().map(|()| vec![])
    }
}
