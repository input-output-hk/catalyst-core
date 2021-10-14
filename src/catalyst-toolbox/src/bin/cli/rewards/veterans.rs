use super::Error;
use catalyst_toolbox::rewards::veterans;
use catalyst_toolbox::rewards::veterans::VeteranAdvisorReward;
use catalyst_toolbox::utils::csv;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VeteransRewards {
    reviews: PathBuf,
    /// Reward (in LOVELACE) to be distributed
    #[structopt(long = "total-rewards")]
    total_rewards: VeteranAdvisorReward,
}

impl VeteransRewards {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            reviews,
            total_rewards,
        } = self;
        let reviews: Vec<veterans::VeteranReview> = csv::load_data_from_csv(&reviews)?;
        let results = veterans::calculate_veteran_advisors_rewards(&reviews, total_rewards);
        Ok(())
    }
}
