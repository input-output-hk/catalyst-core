use crate::load::config::ArtificialUserRequestType as RequestType;
use crate::load::request_generators::jormungandr::SettingsRequestGen;
use crate::load::request_generators::AccountRequestGen;
use crate::load::request_generators::BatchWalletRequestGen;
use crate::load::ServicingStationRequestGen;
use jortestkit::load::Id;
use jortestkit::load::Request;
use jortestkit::load::RequestFailure;
use jortestkit::load::RequestGenerator;
use std::time::Instant;

const DEFAULT_MAX_SPLITS: usize = 7; // equals to 128 splits, will likely not reach that value but it's there just to prevent a stack overflow

pub struct ArtificialUserRequestGen {
    account_gen: Option<AccountRequestGen>,
    static_gen: Option<ServicingStationRequestGen>,
    node_gen: Option<BatchWalletRequestGen>,
    settings_gen: Option<SettingsRequestGen>,
    request_type: RequestType,
    max_splits: usize, // avoid infinite splitting
}

impl ArtificialUserRequestGen {
    pub fn new_account(account_gen: AccountRequestGen) -> Self {
        Self {
            account_gen: Some(account_gen),
            static_gen: None,
            node_gen: None,
            settings_gen: None,
            request_type: RequestType::Account,
            max_splits: DEFAULT_MAX_SPLITS,
        }
    }

    pub fn new_static(static_gen: ServicingStationRequestGen, request_type: RequestType) -> Self {
        Self {
            account_gen: None,
            static_gen: Some(static_gen),
            node_gen: None,
            settings_gen: None,
            request_type,
            max_splits: DEFAULT_MAX_SPLITS,
        }
    }

    pub fn new_settings(settings_gen: SettingsRequestGen) -> Self {
        Self {
            account_gen: None,
            static_gen: None,
            node_gen: None,
            settings_gen: Some(settings_gen),
            request_type: RequestType::Settings,
            max_splits: DEFAULT_MAX_SPLITS,
        }
    }

    pub fn new_node(node_gen: BatchWalletRequestGen) -> Self {
        Self {
            account_gen: None,
            static_gen: None,
            settings_gen: None,
            node_gen: Some(node_gen),
            request_type: RequestType::Vote,
            max_splits: DEFAULT_MAX_SPLITS,
        }
    }

    fn next_request(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        match self.request_type {
            RequestType::Vote => self
                .node_gen
                .as_mut()
                .unwrap()
                .random_votes()
                .map_err(|e| RequestFailure::General(format!("{e:?}"))),
            RequestType::Account => self
                .account_gen
                .as_mut()
                .unwrap()
                .random_account_request()
                .map(|()| vec![None]),
            RequestType::Settings => self
                .settings_gen
                .as_mut()
                .unwrap()
                .settings_request()
                .map(|()| vec![None]),
            _ => self.static_gen.as_mut().unwrap().next_request(),
        }
    }
}

impl RequestGenerator for ArtificialUserRequestGen {
    fn next(&mut self) -> Result<Request, RequestFailure> {
        let start = Instant::now();
        match self.next_request() {
            Ok(v) => Ok(Request {
                ids: v,
                duration: start.elapsed(),
            }),
            Err(e) => Err(RequestFailure::General(format!("{e:?}"))),
        }
    }

    fn split(mut self) -> (Self, Option<Self>) {
        if self.max_splits == 0 {
            return (self, None);
        }

        self.max_splits -= 1;

        let (account_left, account_right) = {
            if let Some(account_gen) = self.account_gen {
                let (left, right) = account_gen.split();
                (Some(left), right)
            } else {
                (self.account_gen, None)
            }
        };

        let (static_gen_left, static_gen_right) = {
            if let Some(static_gen) = self.static_gen {
                let (left, right) = static_gen.split();
                (Some(left), right)
            } else {
                (self.static_gen, None)
            }
        };

        let (node_gen_left, node_gen_right) = {
            if let Some(node_gen) = self.node_gen {
                let (left, right) = node_gen.split();
                (Some(left), right)
            } else {
                (self.node_gen, None)
            }
        };

        let (settings_gen_left, settings_gen_right) = {
            if let Some(settings_gen) = self.settings_gen {
                let (left, right) = settings_gen.split();
                (Some(left), right)
            } else {
                (self.settings_gen, None)
            }
        };

        let other = Self {
            account_gen: account_right,
            static_gen: static_gen_right,
            settings_gen: settings_gen_right,
            node_gen: node_gen_right,
            request_type: self.request_type.clone(),
            max_splits: self.max_splits,
        };

        self.account_gen = account_left;
        self.static_gen = static_gen_left;
        self.node_gen = node_gen_left;
        self.settings_gen = settings_gen_left;

        (self, Some(other))
    }
}
