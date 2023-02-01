use crate::Error;
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

pub struct TxBuilder<P: Payload> {
    builder: wallet::TransactionBuilder<P, AccountSecretKey, WitnessAccountData, account::Witness>,
}

impl<P: Payload> TxBuilder<P> {
    pub fn new(settings: Settings, payload: P) -> Self {
        let builder = wallet::TransactionBuilder::new(settings, payload, BlockDate::first());
        Self { builder }
    }

    pub fn prepare_tx(
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

    pub fn get_sign_data(&self) -> Result<WitnessAccountData, Error> {
        let data = self
            .builder
            .get_sign_data()
            .map_err(|e| Error::wallet_transaction().with(e))?;
        // as inside build_tx() function has been inserted only 1 input, valid tx should contains only first witness sign data
        data.into_iter().next().ok_or_else(Error::invalid_fragment)
    }

    pub fn build_tx(
        self,
        auth: P::Auth,
        signature: &[u8],
        fragment_build_fn: impl FnOnce(Transaction<P>) -> Fragment,
    ) -> Result<Fragment, Error> {
        // as inside build_tx() function has been inserted only 1 input, we should put only 1 witness input
        let witness_input = vec![WitnessInput::Signature(
            account::Witness::from_binary(signature).unwrap(),
        )];
        Ok(fragment_build_fn(
            self.builder
                .finalize_tx(auth, witness_input)
                .map_err(|e| Error::wallet_transaction().with(e))?,
        ))
    }

    pub fn sign_tx(
        self,
        auth: P::Auth,
        account_bytes: &[u8],
        fragment_build_fn: impl FnOnce(Transaction<P>) -> Fragment,
    ) -> Result<Fragment, Error> {
        let account = EitherAccount::new_from_key(
            SecretKey::from_binary(account_bytes)
                .map_err(|e| Error::wallet_transaction().with(e))?,
        );
        // as inside build_tx() function has been inserted only 1 input, we should put only 1 witness input
        let witness_input = vec![WitnessInput::SecretKey(account.secret_key())];
        Ok(fragment_build_fn(
            self.builder
                .finalize_tx(auth, witness_input)
                .map_err(|e| Error::wallet_transaction().with(e))?,
        ))
    }
}
