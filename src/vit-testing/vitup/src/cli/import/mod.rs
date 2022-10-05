mod ideascale;

pub use ideascale::{Error as ImportError, ImportFromIdeascaleFormatCommand};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub enum ImportCommand {
    Ideascale(ImportFromIdeascaleFormatCommand),
}

impl ImportCommand {
    pub fn exec(self) -> Result<(), ImportError> {
        match self {
            Self::Ideascale(ideascale) => ideascale.exec(),
        }
    }
}
