use chain_crypto::SecretKey;
use chain_impl_mockchain::{
    account::SpendingCounter,
    fragment::Fragment,
    header::BlockDate,
    transaction::{Input, Payload, Transaction},
};
use std::{collections::HashMap, str::FromStr};
use wallet::{AccountId, EitherAccount, Settings};

use crate::Error;

pub struct TxBuilder<P: Payload> {
    builder: wallet::TransactionBuilder<P>,
    inputs: HashMap<AccountId, (Input, SpendingCounter)>,
}

impl<P: Payload> TxBuilder<P> {
    pub fn new(settings: Settings, valid_until: BlockDate, payload: P) -> Self {
        let builder = wallet::TransactionBuilder::new(settings, payload, valid_until);
        Self {
            builder,
            inputs: HashMap::new(),
        }
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
        self.inputs
            .insert(account_id, (input.clone(), spending_counter));
        Ok(self)
    }

    pub fn sign_tx(mut self, account_bytes: &[u8]) -> Result<Self, Error> {
        let account = EitherAccount::new_from_key(
            SecretKey::from_binary(account_bytes)
                .map_err(|e| Error::wallet_transaction().with(e))?,
        );

        let (input, spending_counter) = self.inputs.get(&account.account_id()).ok_or(Error::invalid_input("Cannot find corresponded input to the provided account, make sure that you have correctly execute build_tx function first"))?.clone();
        let witness_builder = account.witness_builder(spending_counter);
        self.builder.add_input(input, witness_builder);
        Ok(self)
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
