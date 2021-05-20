mod tally;
mod votes;

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Recover {
    Tally(tally::Replay),
    VotesPrintout(votes::VotesPrintout),
}

impl Recover {
    pub fn exec(self) -> Result<(), tally::Error> {
        match self {
            Recover::Tally(cmd) => cmd.exec(),
            Recover::VotesPrintout(cmd) => cmd.exec(),
        }
    }
}
