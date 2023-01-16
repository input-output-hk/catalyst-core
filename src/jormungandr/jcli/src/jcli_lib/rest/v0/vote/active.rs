use super::{committees::Committees, plans::Plans};
use crate::jcli_lib::rest::Error;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Active {
    /// Committee members
    #[clap(subcommand)]
    Committees(Committees),
    /// Active vote plans
    #[clap(subcommand)]
    Plans(Plans),
}

impl Active {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Active::Committees(committees) => committees.exec(),
            Active::Plans(plans) => plans.exec(),
        }
    }
}
