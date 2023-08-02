use jortestkit::load::{Request, RequestFailure, RequestGenerator};
use std::time::Instant;
use valgrind::WalletNodeRestClient;

const DEFAULT_MAX_SPLITS: usize = 20;

pub struct SettingsRequestGen {
    rest: WalletNodeRestClient,
    max_splits: usize, // avoid infinite splitting
}

impl SettingsRequestGen {
    pub fn new(rest: WalletNodeRestClient) -> Self {
        Self {
            rest,
            max_splits: DEFAULT_MAX_SPLITS,
        }
    }

    pub fn settings_request(&mut self) -> Result<(), RequestFailure> {
        self.rest
            .settings()
            .map_err(|e| RequestFailure::General(format!("{e:?}")))?;
        Ok(())
    }
}

impl RequestGenerator for SettingsRequestGen {
    fn next(&mut self) -> Result<Request, RequestFailure> {
        let start = Instant::now();
        match self.settings_request() {
            Ok(()) => Ok(Request {
                ids: vec![],
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

        let other = Self {
            rest: self.rest.clone(),
            max_splits: self.max_splits,
        };

        (self, Some(other))
    }
}
