mod ideascale;

pub use ideascale::{Error as ImportError, ImportFromIdeascaleFormatCommand};

use clap::Parser;

#[derive(Parser, Debug)]
pub enum ImportCommand {
    #[clap(subcommand)]
    Ideascale(ImportFromIdeascaleFormatCommand),
}

impl ImportCommand {
    pub fn exec(self) -> Result<(), ImportError> {
        match self {
            Self::Ideascale(ideascale) => ideascale.exec(),
        }
    }
}
