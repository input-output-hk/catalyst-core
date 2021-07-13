use crate::backend::WalletNodeRestClient;
use crate::Wallet;
use jortestkit::load::{Id, RequestFailure, RequestGenerator};
use rand::Rng;
use rand_core::OsRng;

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
        let wallet = self
            .wallets
            .get(wallet_index)
            .ok_or(RequestFailure::General(format!(
                "wallet with index '{}' not found",
                wallet_index
            )))?;
        self.rest
            .account_state(wallet.id())
            .map_err(|e| RequestFailure::General(format!("{:?}", e)))?;
        Ok(())
    }
}

impl RequestGenerator for AccountRequestGen {
    fn next(&mut self) -> Result<Vec<Option<Id>>, RequestFailure> {
        self.random_account_request().map(|()| vec![None])
    }
}
