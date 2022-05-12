use super::Error;
use catalyst_toolbox::community_advisors::models::VeteranRankingRow;
use catalyst_toolbox::rewards::veterans::{self, VcaRewards, VeteranAdvisorIncentive};
use catalyst_toolbox::rewards::Rewards;
use catalyst_toolbox::utils::csv;
use rust_decimal::Decimal;
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

    /// Minimum number of rankings for each vca to be considered for reputation and rewards
    /// distribution
    #[structopt(long)]
    min_rankings: usize,

    /// Cutoff for monetary rewards: ranking more reviews than this limit will not result in more rewards
    #[structopt(long)]
    max_rankings_rewards: usize,

    /// Cutoff for reputation: ranking more reviews than this limit will not result in more reputation awarded
    #[structopt(long)]
    max_rankings_reputation: usize,

    /// Agreement rate cutoff list for rewards. These are expected to be provided in descending
    /// order (numerically speaking). The first cutoff that is lower or equal than the computed
    /// agreement rate for each vca is used to select the corresponding modifier (the one that is
    /// in the same position).
    #[structopt(long, required = true)]
    rewards_agreement_rate_cutoffs: Vec<Decimal>,

    /// Cutoff multipliers for rewards: Expected the same number of values than
    /// rewards_agreement_rate_cutoff. The order of these matters, and is expected to be in a 1 to
    /// 1 correspondence with the ones provided in rewards_agreement_rate_cutoffs, meaning than if
    /// the first cutoff is selected then the first modifier is used.
    #[structopt(long, required = true)]
    rewards_agreement_rate_modifiers: Vec<Decimal>,

    /// Agreement rate cutoff list for reputation. These are expected to be provided in descending
    /// order (numerically speaking). The first cutoff that is lower or equal than the computed
    /// agreement rate for each vca is used to select the corresponding modifier (the one that is in
    /// the same position).
    #[structopt(long, required = true)]
    reputation_agreement_rate_cutoffs: Vec<Decimal>,

    /// Cutoff multipliers for reputation: Expected the same number of values than
    /// rewards_agreement_rate_cutoff. The order of these matters, and is expected to be in a 1 to
    /// 1 correspondence with the ones provided in reputation_agreement_rate_cutoffs, meaning than
    /// if the first cutoff is selected then the first modifier is used.
    #[structopt(long, required = true)]
    reputation_agreement_rate_modifiers: Vec<Decimal>,
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
            rewards_agreement_rate_cutoffs,
            rewards_agreement_rate_modifiers,
            reputation_agreement_rate_cutoffs,
            reputation_agreement_rate_modifiers,
        } = self;
        let reviews: Vec<VeteranRankingRow> = csv::load_data_from_csv::<_, b','>(&from)?;

        if rewards_agreement_rate_cutoffs.len() != rewards_agreement_rate_modifiers.len() {
            return Err(Error::InvalidInput(
                "Expected same number of rewards_agreement_rate_cutoffs and rewards_agreement_rate_modifiers"
                    .to_string(),
            ));
        }

        if reputation_agreement_rate_cutoffs.len() != reputation_agreement_rate_modifiers.len() {
            return Err(Error::InvalidInput(
                "Expected same number of reputation_agreement_rate_cutoffs and reputation_agreement_rate_modifiers"
                    .to_string(),
            ));
        }

        if !is_descending(&rewards_agreement_rate_cutoffs) {
            return Err(Error::InvalidInput(
                "Expected rewards_agreement_rate_cutoffs to be descending".to_string(),
            ));
        }

        if !is_descending(&reputation_agreement_rate_cutoffs) {
            return Err(Error::InvalidInput(
                "Expected rewards_agreement_rate_cutoffs to be descending".to_string(),
            ));
        }

        let results = veterans::calculate_veteran_advisors_incentives(
            &reviews,
            total_rewards,
            min_rankings..=max_rankings_rewards,
            min_rankings..=max_rankings_reputation,
            rewards_agreement_rate_cutoffs
                .into_iter()
                .zip(rewards_agreement_rate_modifiers.into_iter())
                .collect(),
            reputation_agreement_rate_cutoffs
                .into_iter()
                .zip(reputation_agreement_rate_modifiers.into_iter())
                .collect(),
        );

        csv::dump_data_to_csv(rewards_to_csv_data(results).iter(), &to).unwrap();

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

fn is_descending(v: &Vec<Decimal>) -> bool {
    let mut clone = v.clone();
    clone.sort_by(|a, b| b.cmp(a));

    v == &clone
}
