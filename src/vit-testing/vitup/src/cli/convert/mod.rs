mod ideascale;

pub use ideascale::ConvertFromIdeascale;

use crate::Result;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub enum ConvertCommand {
    Ideascale(ConvertFromIdeascale),
}

impl ConvertCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Ideascale(ideascale) => ideascale.exec(),
        }
    }
}
