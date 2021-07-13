use crate::load::config::ArtificialUserRequestType as RequestType;
use crate::load::request_generators::AccountRequestGen;
use crate::load::request_generators::BatchWalletRequestGen;
use crate::load::ServicingStationRequestGen;
use jortestkit::load::Id;
use jortestkit::load::RequestFailure;
use jortestkit::load::RequestGenerator;

pub struct ArtificialUserRequestGen {
    account_gen: Option<AccountRequestGen>,
    static_gen: Option<ServicingStationRequestGen>,
    node_gen: Option<BatchWalletRequestGen>,
    request_type: RequestType,
}

impl ArtificialUserRequestGen {
    pub fn new_account(account_gen: AccountRequestGen) -> Self {
        Self {
            account_gen: Some(account_gen),
            static_gen: None,
            node_gen: None,
            request_type: RequestType::Account,
        }
    }

    pub fn new_static(static_gen: ServicingStationRequestGen, request_type: RequestType) -> Self {
        Self {
            account_gen: None,
            static_gen: Some(static_gen),
            node_gen: None,
            request_type,
        }
    }

    pub fn new_node(node_gen: BatchWalletRequestGen) -> Self {
        Self {
            account_gen: None,
            static_gen: None,
            node_gen: Some(node_gen),
            request_type: RequestType::Vote,
        }
    }

    fn next_request(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        match self.request_type {
            RequestType::Vote => self
                .node_gen
                .as_mut()
                .unwrap()
                .random_votes()
                .map_err(|e| RequestFailure::General(format!("{:?}", e))),
            RequestType::Account => self
                .account_gen
                .as_mut()
                .unwrap()
                .random_account_request()
                .map(|()| vec![None]),
            _ => self.static_gen.as_mut().unwrap().next_request(),
        }
    }
}

impl RequestGenerator for ArtificialUserRequestGen {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        self.next_request()
    }
}
