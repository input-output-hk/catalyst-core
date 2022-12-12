use chain_impl_mockchain::{
    account::SpendingCounter,
    fragment::Fragment,
    header::BlockDate,
    transaction::{Input, Payload, Transaction},
};
use wallet::{EitherAccount, Settings};

use crate::Error;

pub struct TxBuilder<'settings, P: Payload> {
    builder: wallet::TransactionBuilder<'settings, P>,
}

impl<'settings, P: Payload> TxBuilder<'settings, P> {
    pub fn new(settings: &'settings Settings, valid_until: BlockDate, payload: P) -> Self {
        let builder = wallet::TransactionBuilder::new(settings, payload, valid_until);
        Self { builder }
    }

    pub fn build_tx(mut self, account: EitherAccount, spending_counter: SpendingCounter) -> Self {
        // It is needed to provide a 1 extra input as we are generating it later, but should take into account at this place.
        let value = self.builder.estimate_fee_with(1, 0);
        let input = Input::from_account_public_key(account.account_id().into(), value);
        let witness_builder = account.witness_builder(spending_counter);
        self.builder.add_input(input, witness_builder);
        self
    }

    pub fn finalize_tx(
        self,
        auth: P::Auth,
        fragment_build_fn: impl FnOnce(Transaction<P>) -> Fragment,
    ) -> Result<Fragment, Error> {
        Ok(fragment_build_fn(
            self.builder
                .finalize_tx(auth)
                .map_err(|e| Error::wallet_transaction().with(e))?,
        ))
    }
}
