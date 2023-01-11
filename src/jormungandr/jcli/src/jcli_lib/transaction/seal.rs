use crate::jcli_lib::transaction::{common, Error};
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Seal {
    #[clap(flatten)]
    pub common: common::CommonTransaction,
}

impl Seal {
    pub fn exec(self) -> Result<(), Error> {
        let mut transaction = self.common.load()?;
        transaction.seal()?;
        self.common.store(&transaction)
    }
}
