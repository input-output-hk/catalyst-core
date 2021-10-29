mod community_advisors;
mod veterans;
mod voters;

use structopt::StructOpt;
use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("error while writing to csv")]
    Csv(#[from] csv::Error),

    #[error("requested funds cannot be parsed: {0}")]
    InvalidRequestedFunds(String),

    #[error(transparent)]
    Other(#[from] jcli_lib::jcli_lib::block::Error),

    #[error("{0}")]
    InvalidInput(String),
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Rewards {
    /// Calculate rewards for voters base on their stake
    Voters(voters::VotersRewards),

    /// Calculate community advisors rewards
    CommunityAdvisors(community_advisors::CommunityAdvisors),

    /// Calculate rewards for veteran community advisors
    Veterans(veterans::VeteransRewards),
}

impl Rewards {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Rewards::Voters(cmd) => cmd.exec(),
            Rewards::CommunityAdvisors(cmd) => cmd.exec(),
            Rewards::Veterans(cmd) => cmd.exec(),
        }
    }
}
