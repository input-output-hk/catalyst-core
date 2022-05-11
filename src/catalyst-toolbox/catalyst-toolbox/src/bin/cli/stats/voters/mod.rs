mod active;
mod initials;

use active::ActiveVotersCommand;
use initials::InitialVotersCommand;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum VotersCommand {
    Initials(InitialVotersCommand),
    Active(ActiveVotersCommand),
}

impl VotersCommand {
    pub fn exec(self) -> Result<(), catalyst_toolbox::stats::Error> {
        match self {
            Self::Initials(initials) => initials.exec(),
            Self::Active(active) => active.exec(),
        }
    }
}
