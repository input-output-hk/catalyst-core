use crate::jcli_lib::transaction::{common, Error};
use jormungandr_lib::interfaces::Certificate;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct AddCertificate {
    #[clap(flatten)]
    pub common: common::CommonTransaction,

    /// bech32-encoded certificate
    pub certificate: Certificate,
}

impl AddCertificate {
    pub fn exec(self) -> Result<(), Error> {
        let mut transaction = self.common.load()?;
        transaction.set_extra(self.certificate)?;
        self.common.store(&transaction)
    }
}
