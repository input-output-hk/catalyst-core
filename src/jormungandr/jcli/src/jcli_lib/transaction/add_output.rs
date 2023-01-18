use crate::jcli_lib::transaction::{common, Error};
use chain_impl_mockchain::transaction::Output;
use clap::Parser;
use jormungandr_lib::interfaces;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct AddOutput {
    #[clap(flatten)]
    pub common: common::CommonTransaction,

    /// the UTxO address or account address to credit funds to
    #[clap(name = "ADDRESS")]
    pub address: interfaces::Address,

    /// the value
    #[clap(name = "VALUE")]
    pub value: interfaces::Value,
}

impl AddOutput {
    pub fn exec(self) -> Result<(), Error> {
        let mut transaction = self.common.load()?;

        transaction.add_output(Output {
            address: self.address.into(),
            value: self.value.into(),
        })?;

        self.common.store(&transaction)
    }
}
