use crate::backend::VitStationRestClient;
use crate::load::config::ServicingStationRequestType as RequestType;
use crate::rand::RngCore;
use crate::Proposal;
use jortestkit::load::{Id, RequestFailure, RequestGenerator};
use rand::rngs::OsRng;

pub struct ServicingStationRequestGen {
    pub client: VitStationRestClient,
    pub proposals: Vec<Proposal>,
    pub rand: OsRng,
    pub request_type: RequestType,
}

impl ServicingStationRequestGen {
    pub fn new_fund(client: VitStationRestClient) -> Self {
        Self {
            client,
            proposals: Vec::new(),
            rand: OsRng,
            request_type: RequestType::Fund,
        }
    }

    pub fn new_challenges(client: VitStationRestClient) -> Self {
        Self {
            client,
            proposals: Vec::new(),
            rand: OsRng,
            request_type: RequestType::Challenges,
        }
    }

    pub fn new_proposals(client: VitStationRestClient) -> Self {
        Self {
            client,
            proposals: Vec::new(),
            rand: OsRng,
            request_type: RequestType::Proposals,
        }
    }

    pub fn new_proposal(client: VitStationRestClient, proposals: Vec<Proposal>) -> Self {
        Self {
            client,
            proposals,
            rand: OsRng,
            request_type: RequestType::Proposal,
        }
    }

    fn next_usize(&mut self) -> usize {
        self.rand.next_u32() as usize
    }

    fn random_proposal(&mut self) -> Result<reqwest::blocking::Response, RequestFailure> {
        let index = self.next_usize() % self.proposals.len();
        let proposal = self.proposals.get(index).unwrap().clone();
        self.client
            .proposal_raw(&proposal.internal_id.to_string())
            .map_err(|e| RequestFailure::General(format!("{:?}", e)))
    }

    pub fn next_request(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        match self.request_type {
            RequestType::Fund => self
                .client
                .funds_raw()
                .map(|_response| vec![None])
                .map_err(|e| RequestFailure::General(format!("{:?}", e))),
            RequestType::Challenges => self
                .client
                .challenges_raw()
                .map(|_response| vec![None])
                .map_err(|e| RequestFailure::General(format!("{:?}", e))),
            RequestType::Proposal => self.random_proposal().map(|_response| vec![None]),
            RequestType::Proposals => self
                .client
                .proposals_raw()
                .map(|_response| vec![None])
                .map_err(|e| RequestFailure::General(format!("{:?}", e))),
        }
    }
}

impl RequestGenerator for ServicingStationRequestGen {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        self.next_request()
    }
}
