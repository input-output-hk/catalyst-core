use crate::Wallet;
use jortestkit::load::{Request, RequestFailure, RequestGenerator};
use rand::Rng;
use rand_core::OsRng;
use std::time::Instant;
use valgrind::WalletNodeRestClient;

pub struct AccountRequestGen {
    rand: OsRng,
    wallets: Vec<Wallet>,
    rest: WalletNodeRestClient,
}

impl AccountRequestGen {
    pub fn new(wallets: Vec<Wallet>, rest: WalletNodeRestClient) -> Self {
        Self {
            wallets,
            rand: OsRng,
            rest,
        }
    }

    pub fn random_account_request(&mut self) -> Result<(), RequestFailure> {
        let wallet_index = self.rand.gen_range(0..self.wallets.len());
        let wallet = self.wallets.get(wallet_index).ok_or_else(|| {
            RequestFailure::General(format!("wallet with index '{wallet_index}' not found"))
        })?;
        self.rest
            .account_state(wallet.id())
            .map_err(|e| RequestFailure::General(format!("{e:?}")))?;
        Ok(())
    }
}

impl RequestGenerator for AccountRequestGen {
    fn next(&mut self) -> Result<Request, RequestFailure> {
        let start = Instant::now();
        match self.random_account_request() {
            Ok(()) => Ok(Request {
                ids: vec![],
                duration: start.elapsed(),
            }),
            Err(e) => Err(RequestFailure::General(format!("{e:?}"))),
        }
    }

    fn split(mut self) -> (Self, Option<Self>) {
        let wallets_len = self.wallets.len();
        if wallets_len <= 1 {
            return (self, None);
        }
        let wallets = self.wallets.split_off(wallets_len / 2);
        let new_gen = Self {
            rand: self.rand,
            wallets,
            rest: self.rest.clone(),
        };

        (self, Some(new_gen))
    }
}
