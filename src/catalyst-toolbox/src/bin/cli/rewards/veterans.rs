use super::Error;
use catalyst_toolbox::rewards::veterans;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VeteransRewards {
    reviews: PathBuf,
    /// Reward (in LOVELACE) to be distributed
    #[structopt(long = "total-rewards")]
    total_rewards: u64,
}

impl VeteransRewards {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            reviews,
            total_rewards,
        };

        Ok(())
    }
}
