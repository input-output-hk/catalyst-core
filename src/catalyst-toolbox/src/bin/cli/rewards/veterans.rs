use super::Error;
use catalyst_toolbox::rewards::veterans;
use catalyst_toolbox::rewards::veterans::VeteranAdvisorReward;
use catalyst_toolbox::utils::csv;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VeteransRewards {
    /// Reviews csv file path
    reviews: PathBuf,

    /// Results file output path
    output: PathBuf,

    /// Reward to be distributed
    #[structopt(long = "total-rewards")]
    total_rewards: VeteranAdvisorReward,
}

impl VeteransRewards {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            reviews,
            output,
            total_rewards,
        } = self;
        let reviews: Vec<veterans::VeteranReviewsCount> = csv::load_data_from_csv(&reviews)?;
        let results = veterans::calculate_veteran_advisors_rewards(&reviews, total_rewards);
        csv::dump_data_to_csv(&results, &output)?;

        Ok(())
    }
}
