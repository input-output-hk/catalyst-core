mod community_advisors;
mod full;
mod veterans;
mod voters;

use std::path::PathBuf;

use color_eyre::Report;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Rewards {
    /// Calculate rewards for voters base on their stake
    Voters(voters::VotersRewards),

    /// Calculate community advisors rewards
    CommunityAdvisors(community_advisors::CommunityAdvisors),

    /// Calculate rewards for veteran community advisors
    Veterans(veterans::VeteransRewards),

    /// Calculate full rewards based on a config file
    Full { path: PathBuf },
}

impl Rewards {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Rewards::Voters(cmd) => cmd.exec(),
            Rewards::CommunityAdvisors(cmd) => cmd.exec(),
            Rewards::Veterans(cmd) => cmd.exec(),
            Rewards::Full { path } => full::full_rewards(&path),
        }
    }
}
