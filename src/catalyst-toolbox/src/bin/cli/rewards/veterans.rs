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
    from: PathBuf,

    /// Results file output path
    to: PathBuf,

    /// Reward to be distributed
    #[structopt(long = "total-rewards")]
    total_rewards: VeteranAdvisorReward,
}

impl VeteransRewards {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            from,
            to,
            total_rewards,
        } = self;
        let reviews: Vec<veterans::VeteranReviewsCount> = csv::load_data_from_csv(&from)?;
        let results = veterans::calculate_veteran_advisors_rewards(&reviews, total_rewards);
        csv::dump_data_to_csv(&results, &to)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::rewards::veterans::VeteransRewards;
    use catalyst_toolbox::rewards::veterans::VeteranAdvisorReward;
    use jcli_lib::utils::io;
    use rust_decimal::prelude::FromStr;
    use std::io::BufRead;

    #[test]
    fn test_output_csv() {
        let resource_input = "./resources/testing/veteran_reviews_count.csv";
        let tmp_file = assert_fs::NamedTempFile::new("outfile.csv").unwrap();

        let export = VeteransRewards {
            from: resource_input.into(),
            to: tmp_file.path().into(),
            total_rewards: 1000.into(),
        };

        export.exec().unwrap();
        let reader = io::open_file_read(&Some(tmp_file.path())).unwrap();
        let expected_reward = VeteranAdvisorReward::from(200);
        for line in reader.lines() {
            let line = line.unwrap();
            let res: Vec<&str> = line.split(',').collect();
            let reward = VeteranAdvisorReward::from_str(res[1]).unwrap();
            assert_eq!(reward, expected_reward);
        }
    }
}
