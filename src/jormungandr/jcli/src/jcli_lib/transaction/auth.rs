use crate::jcli_lib::{
    certificate::read_input,
    transaction::{common, Error},
};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Auth {
    #[clap(flatten)]
    pub common: common::CommonTransaction,
    /// path to the file with the signing key
    #[clap(short = 'k', long = "key")]
    pub signing_keys: Vec<PathBuf>,
}

impl Auth {
    pub fn exec(self) -> Result<(), Error> {
        let mut transaction = self.common.load()?;

        if self.signing_keys.is_empty() {
            return Err(Error::NoSigningKeys);
        }

        let keys_str: Result<Vec<String>, Error> = self
            .signing_keys
            .iter()
            .map(|sk| {
                read_input(Some(sk.as_ref())).map_err(|e| Error::CertificateError { error: e })
            })
            .collect();
        let keys_str = keys_str?;

        transaction.set_auth(&keys_str)?;

        self.common.store(&transaction)
    }
}
