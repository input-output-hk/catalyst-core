use crate::backend::VitStationRestClient;
use crate::load::config::ServicingStationRequestType as RequestType;
use crate::rand::RngCore;
use crate::Proposal;
use jortestkit::load::{Id, Request, RequestFailure, RequestGenerator};
use rand::rngs::OsRng;
use std::time::Instant;

const DEFAULT_MAX_SPLITS: usize = 7; // equals to 128 splits, will likely not reach that value but it's there just to prevent a stack overflow

pub struct ServicingStationRequestGen {
    client: VitStationRestClient,
    proposals: Vec<Proposal>,
    rand: OsRng,
    request_type: RequestType,
    max_splits: usize, // avoid infinite splitting
}

impl ServicingStationRequestGen {
    pub fn new_fund(client: VitStationRestClient) -> Self {
        Self {
            client,
            proposals: Vec::new(),
            rand: OsRng,
            request_type: RequestType::Fund,
            max_splits: DEFAULT_MAX_SPLITS,
        }
    }

    pub fn new_challenges(client: VitStationRestClient) -> Self {
        Self {
            client,
            proposals: Vec::new(),
            rand: OsRng,
            request_type: RequestType::Challenges,
            max_splits: DEFAULT_MAX_SPLITS,
        }
    }

    pub fn new_proposals(client: VitStationRestClient) -> Self {
        Self {
            client,
            proposals: Vec::new(),
            rand: OsRng,
            request_type: RequestType::Proposals,
            max_splits: DEFAULT_MAX_SPLITS,
        }
    }

    pub fn new_proposal(client: VitStationRestClient, proposals: Vec<Proposal>) -> Self {
        Self {
            client,
            proposals,
            rand: OsRng,
            request_type: RequestType::Proposal,
            max_splits: DEFAULT_MAX_SPLITS,
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
    fn next(&mut self) -> Result<Request, RequestFailure> {
        let start = Instant::now();
        match self.next_request() {
            Ok(v) => Ok(Request {
                ids: v,
                duration: start.elapsed(),
            }),
            Err(e) => Err(RequestFailure::General(format!("{:?}", e))),
        }
    }

    fn split(mut self) -> (Self, Option<Self>) {
        if self.max_splits == 0 {
            return (self, None);
        }

        self.max_splits -= 1;

        let other = Self {
            client: self.client.clone(),
            proposals: self.proposals.clone(),
            rand: OsRng,
            request_type: self.request_type.clone(),
            max_splits: self.max_splits,
        };
        (self, Some(other))
    }
}
