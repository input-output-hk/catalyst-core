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

fn set_verbosity(verbosity: usize) {
    if verbosity > 0 {
        std::env::set_var(
            "RUST_LOG",
            match verbosity {
                0 => unreachable!(),
                1 => "warn",
                2 => "info",
                3 => "debug",
                _ => "trace",
            },
        )
    }
}
