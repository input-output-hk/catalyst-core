use crate::jcli_lib::transaction::{common, Error};
use jormungandr_lib::interfaces;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct SetExpiryDate {
    #[clap(flatten)]
    pub common: common::CommonTransaction,

    /// the slot this transaction should be valid until, for example 3.14
    #[clap(name = "BLOCKDATE")]
    pub valid_until: interfaces::BlockDate,
}

impl SetExpiryDate {
    pub fn exec(self) -> Result<(), Error> {
        let mut transaction = self.common.load()?;
        transaction.set_expiry_date(self.valid_until)?;
        self.common.store(&transaction)
    }
}
