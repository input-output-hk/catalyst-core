mod tally;
mod votes;

use color_eyre::Report;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Recover {
    Tally(tally::ReplayCli),
    VotesPrintout(votes::VotesPrintout),
}

impl Recover {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Recover::Tally(cmd) => cmd.exec(),
            Recover::VotesPrintout(cmd) => cmd.exec(),
        }
    }
}
