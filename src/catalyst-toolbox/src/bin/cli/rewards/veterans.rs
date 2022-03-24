use super::Error;
use catalyst_toolbox::community_advisors::models::VeteranRankingRow;
use catalyst_toolbox::rewards::veterans::{self, VcaRewards, VeteranAdvisorIncentive};
use catalyst_toolbox::rewards::Rewards;
use catalyst_toolbox::utils::csv;
use serde::Serialize;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct VeteransRewards {
    /// Reviews csv file path
    from: PathBuf,

    /// Results file output path
    to: PathBuf,

    /// Reward to be distributed
    #[structopt(long = "total-rewards")]
    total_rewards: Rewards,

    /// Minimum number of rankings for each vca to be considered for reputatin and rewards
    /// distribution
    #[structopt(long)]
    min_rankings: usize,

    /// Cutoff for monetary rewards: ranking more reviews than this limit will not result in more rewards
    #[structopt(long)]
    max_rankings_rewards: usize,

    /// Cutoff for reputation: ranking more reviews than this limit will not result in more reputation awarded
    #[structopt(long)]
    max_rankings_reputation: usize,
}

impl VeteransRewards {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            from,
            to,
            total_rewards,
            min_rankings,
            max_rankings_reputation,
            max_rankings_rewards,
        } = self;
        let reviews: Vec<VeteranRankingRow> = csv::load_data_from_csv::<_, b','>(&from)?;
        let results = veterans::calculate_veteran_advisors_incentives(
            &reviews,
            total_rewards,
            min_rankings..=max_rankings_rewards,
            min_rankings..=max_rankings_reputation,
        );

        csv::dump_data_to_csv(&rewards_to_csv_data(results), &to).unwrap();

        Ok(())
    }
}

fn rewards_to_csv_data(rewards: VcaRewards) -> Vec<impl Serialize> {
    #[derive(Serialize)]
    struct Entry {
        id: String,
        rewards: Rewards,
        reputation: u64,
    }

    rewards
        .into_iter()
        .map(
            |(
                id,
                VeteranAdvisorIncentive {
                    rewards,
                    reputation,
                },
            )| Entry {
                id,
                rewards,
                reputation,
            },
        )
        .collect()
}
