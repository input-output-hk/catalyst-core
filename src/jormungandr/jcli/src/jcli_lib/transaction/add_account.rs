use crate::jcli_lib::transaction::{common, Error};
use jormungandr_lib::interfaces;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct AddAccount {
    #[clap(flatten)]
    pub common: common::CommonTransaction,

    /// the account to debit the funds from
    #[clap(name = "ACCOUNT")]
    pub account: interfaces::Address,

    /// the value
    #[clap(name = "VALUE")]
    pub value: interfaces::Value,
}

impl AddAccount {
    pub fn exec(self) -> Result<(), Error> {
        let mut transaction = self.common.load()?;
        transaction.add_account(self.account, self.value)?;
        self.common.store(&transaction)
    }
}
