mod community_advisors;
mod full;
mod proposers;
mod veterans;
mod voters;

use std::path::PathBuf;

use catalyst_toolbox::{http::default_http_client, rewards::proposers as proposers_lib};
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

    /// Calculate rewards for propsers
    Proposers(proposers_lib::ProposerRewards),
}

impl Rewards {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            Rewards::Voters(cmd) => cmd.exec(),
            Rewards::CommunityAdvisors(cmd) => cmd.exec(),
            Rewards::Veterans(cmd) => cmd.exec(),
            Rewards::Full { path } => full::full_rewards(&path),
            Rewards::Proposers(proposers) => {
                proposers::rewards(&proposers, &default_http_client(None))
            }
        }
    }
}
