pub mod tally;

use structopt::StructOpt;

use crate::cli::recovery::tally::Error;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Recover {
    Tally(tally::Replay),
}

impl Recover {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Recover::Tally(tally) => tally.exec(),
        }
    }
}
