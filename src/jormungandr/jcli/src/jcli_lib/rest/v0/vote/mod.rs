mod active;
mod committees;
mod plans;

use self::active::Active;
use crate::jcli_lib::rest::Error;
use clap::Parser;

#[derive(Parser)]
#[clap(name = "active", rename_all = "kebab-case")]
pub enum Vote {
    /// Active vote related operations
    #[clap(subcommand)]
    Active(Active),
}

impl Vote {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Vote::Active(active) => active.exec(),
        }
    }
}
