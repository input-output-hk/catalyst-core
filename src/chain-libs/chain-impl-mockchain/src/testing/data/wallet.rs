use std::collections::HashMap;

use crate::{
    chaintypes::HeaderId,
    key::EitherEd25519SecretKey,
    testing::data::{AddressData, AddressDataValue},
    tokens::name::TokenName,
    transaction::{Input, Output, TransactionAuthData, Witness},
    value::Value,
};
use chain_addr::{Address, Discrimination};
use chain_crypto::{Ed25519, PublicKey};

#[derive(Clone, Debug)]
pub struct Wallet {
    alias: String,
    account: AddressDataValue,
}

impl Wallet {
    pub fn from_address_data_value_and_alias<S: Into<String>>(
        alias: S,
        account: AddressDataValue,
    ) -> Self {
        Wallet {
            alias: alias.into(),
            account,
        }
    }

    pub fn from_address_data_value(account: AddressDataValue) -> Self {
        Wallet {
            alias: "".to_owned(),
            account,
        }
    }

    pub fn from_value(initial_value: Value) -> Self {
        Wallet::new("", initial_value)
    }

    pub fn new(alias: &str, initial_value: Value) -> Self {
        Wallet {
            alias: alias.to_owned(),
            account: AddressDataValue::account(Discrimination::Test, initial_value),
        }
    }

    pub fn new_with_tokens(
        alias: &str,
        initial_value: Value,
        tokens: HashMap<TokenName, Value>,
    ) -> Self {
        Wallet {
            alias: alias.to_owned(),
            account: AddressDataValue::account_with_tokens(
                Discrimination::Test,
                initial_value,
                tokens,
            ),
        }
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn value(&self) -> Value {
        self.account.value
    }

    pub fn public_key(&self) -> PublicKey<Ed25519> {
        self.account.public_key()
    }

    pub fn private_key(&self) -> EitherEd25519SecretKey {
        self.account.private_key()
    }

    pub fn make_output(&self) -> Output<Address> {
        self.account.make_output()
    }

    pub fn make_output_with_value(&self, value: Value) -> Output<Address> {
        self.account.make_output_with_value(value)
    }

    pub fn make_input_with_value(&self, value: Value) -> Input {
        self.account.make_input_with_value(None, value)
    }

    pub fn as_account(&self) -> AddressDataValue {
        self.account.clone()
    }

    pub fn as_account_data(&self) -> AddressData {
        self.as_account().into()
    }

    pub fn as_address(&self) -> Address {
        self.as_account_data().address
    }

    pub fn confirm_transaction(&mut self) {
        self.confirm_transaction_at_lane(0);
    }

    pub fn confirm_transaction_at_lane(&mut self, lane: usize) {
        self.account.confirm_transaction_at_lane(lane).unwrap();
    }

    pub fn make_witness<'a>(
        &mut self,
        block0_hash: &HeaderId,
        tad: TransactionAuthData<'a>,
    ) -> Witness {
        self.make_witness_at_lane(block0_hash, 0, tad)
    }

    pub fn make_witness_at_lane<'a>(
        &mut self,
        block0_hash: &HeaderId,
        lane: usize,
        tad: TransactionAuthData<'a>,
    ) -> Witness {
        self.as_account()
            .make_witness_with_lane(block0_hash, lane, tad)
    }
}
