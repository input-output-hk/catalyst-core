use chain_crypto::SecretKey;
use chain_impl_mockchain::{
    account::{self, SpendingCounter},
    fragment::Fragment,
    header::BlockDate,
    transaction::{Input, Payload, Transaction, WitnessAccountData},
};
use std::str::FromStr;
use wallet::{
    transaction::{AccountSecretKey, WitnessInput},
    AccountId, AccountWitnessBuilder, EitherAccount, Settings,
};

use crate::Error;

pub struct TxBuilder<P: Payload> {
    builder: wallet::TransactionBuilder<P, AccountSecretKey, WitnessAccountData, account::Witness>,
}

impl<P: Payload> TxBuilder<P> {
    pub fn new(settings: Settings, payload: P) -> Self {
        let builder = wallet::TransactionBuilder::new(settings, payload, BlockDate::first());
        Self { builder }
    }

    pub fn build_tx(
        mut self,
        account_id_hex: String,
        spending_counter: SpendingCounter,
    ) -> Result<Self, Error> {
        let account_id = AccountId::from_str(&account_id_hex)
            .map_err(|e| Error::wallet_transaction().with(e))?;

        // It is needed to provide a 1 extra input as we are generating it later, but should take into account at this place.
        let value = self.builder.estimate_fee_with(1, 0);
        let input = Input::from_account_public_key(account_id.into(), value);
        self.builder
            .add_input(input, AccountWitnessBuilder(spending_counter));
        Ok(self)
    }

    pub fn finalize_tx(
        self,
        auth: P::Auth,
        account_bytes: &[u8],
        fragment_build_fn: impl FnOnce(Transaction<P>) -> Fragment,
    ) -> Result<Fragment, Error> {
        let account = EitherAccount::new_from_key(
            SecretKey::from_binary(account_bytes)
                .map_err(|e| Error::wallet_transaction().with(e))?,
        );
        let inputs_size = self.builder.inputs().len();

        Ok(fragment_build_fn(
            self.builder
                .finalize_tx(
                    auth,
                    vec![WitnessInput::SecretKey(account.secret_key()); inputs_size],
                )
                .map_err(|e| Error::wallet_transaction().with(e))?,
        ))
    }
}
