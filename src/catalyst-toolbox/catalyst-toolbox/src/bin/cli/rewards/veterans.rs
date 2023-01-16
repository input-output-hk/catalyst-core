use catalyst_toolbox::community_advisors::models::VeteranRankingRow;
use catalyst_toolbox::rewards::veterans::{self, VcaRewards, VeteranAdvisorIncentive};
use catalyst_toolbox::utils::csv;
use clap::Parser;
use color_eyre::eyre::{bail, eyre};
use color_eyre::Report;
use rust_decimal::{prelude::*, Decimal};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct VeteransRewards {
    /// Reviews csv file path
    from: PathBuf,

    /// Results file output path
    to: PathBuf,

    /// Reward to be distributed (integer value)
    #[clap(long = "total-rewards")]
    total_rewards: Decimal,

    /// Minimum number of rankings for each vca to be considered for reputation and rewards
    /// distribution
    #[clap(long)]
    min_rankings: usize,

    /// Cutoff for monetary rewards: ranking more reviews than this limit will not result in more rewards
    #[clap(long)]
    max_rankings_rewards: usize,

    /// Cutoff for reputation: ranking more reviews than this limit will not result in more reputation awarded
    #[clap(long)]
    max_rankings_reputation: usize,

    /// Agreement rate cutoff list for rewards. These are expected to be provided in descending
    /// order (numerically speaking). The first cutoff that is lower or equal than the computed
    /// agreement rate for each vca is used to select the corresponding modifier (the one that is
    /// in the same position).
    #[clap(long, required = true)]
    rewards_agreement_rate_cutoffs: Vec<Decimal>,

    /// Cutoff multipliers for rewards: Expected the same number of values than
    /// rewards_agreement_rate_cutoff. The order of these matters, and is expected to be in a 1 to
    /// 1 correspondence with the ones provided in rewards_agreement_rate_cutoffs, meaning than if
    /// the first cutoff is selected then the first modifier is used.
    #[clap(long, required = true)]
    rewards_agreement_rate_modifiers: Vec<Decimal>,

    /// Agreement rate cutoff list for reputation. These are expected to be provided in descending
    /// order (numerically speaking). The first cutoff that is lower or equal than the computed
    /// agreement rate for each vca is used to select the corresponding modifier (the one that is in
    /// the same position).
    #[clap(long, required = true)]
    reputation_agreement_rate_cutoffs: Vec<Decimal>,

    /// Cutoff multipliers for reputation: Expected the same number of values than
    /// rewards_agreement_rate_cutoff. The order of these matters, and is expected to be in a 1 to
    /// 1 correspondence with the ones provided in reputation_agreement_rate_cutoffs, meaning than
    /// if the first cutoff is selected then the first modifier is used.
    #[clap(long, required = true)]
    reputation_agreement_rate_modifiers: Vec<Decimal>,
}

impl VeteransRewards {
    pub fn exec(self) -> Result<(), Report> {
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

        vca_rewards(
            from,
            to,
            rewards_agreement_rate_cutoffs,
            rewards_agreement_rate_modifiers,
            reputation_agreement_rate_cutoffs,
            reputation_agreement_rate_modifiers,
            total_rewards,
            min_rankings,
            max_rankings_reputation,
            max_rankings_rewards,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn vca_rewards(
    reviews_csv: PathBuf,
    output: PathBuf,
    rewards_agreement_rate_cutoffs: Vec<Decimal>,
    rewards_agreement_rate_modifiers: Vec<Decimal>,
    reputation_agreement_rate_cutoffs: Vec<Decimal>,
    reputation_agreement_rate_modifiers: Vec<Decimal>,
    total_rewards: Decimal,
    min_rankings: usize,
    max_rankings_reputation: usize,
    max_rankings_rewards: usize,
) -> Result<(), Report> {
    let reviews: Vec<VeteranRankingRow> = csv::load_data_from_csv::<_, b','>(&reviews_csv)?;

    if rewards_agreement_rate_cutoffs.len() != rewards_agreement_rate_modifiers.len() {
        bail!(
                "Expected same number of rewards_agreement_rate_cutoffs and rewards_agreement_rate_modifiers"
            );
    }

    if reputation_agreement_rate_cutoffs.len() != reputation_agreement_rate_modifiers.len() {
        bail!(
                "Expected same number of reputation_agreement_rate_cutoffs and reputation_agreement_rate_modifiers"
            );
    }

    if !is_descending(&rewards_agreement_rate_cutoffs) {
        bail!("Expected rewards_agreement_rate_cutoffs to be descending");
    }

    if !is_descending(&reputation_agreement_rate_cutoffs) {
        bail!("Expected rewards_agreement_rate_cutoffs to be descending");
    }

    if !is_descending(&rewards_agreement_rate_cutoffs) {
        bail!("Expected rewards_agreement_rate_cutoffs to be descending");
    }

    if !is_descending(&reputation_agreement_rate_cutoffs) {
        bail!("Expected rewards_agreement_rate_cutoffs to be descending");
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

    csv::dump_data_to_csv(rewards_to_csv_data(results).iter(), &output).unwrap();

    Ok(())
}

fn rewards_to_csv_data(rewards: VcaRewards) -> Result<Vec<impl Serialize>, Report> {
    #[derive(Serialize)]
    struct Entry {
        id: String,
        rewards: u64,
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
            )| {
                Ok(Entry {
                    id,
                    rewards: rewards.to_u64().ok_or_else(|| eyre!("Rewards overflow"))?,
                    reputation,
                })
            },
        )
        .collect()
}

fn is_descending(v: &Vec<Decimal>) -> bool {
    let mut clone = v.clone();
    clone.sort_by(|a, b| b.cmp(a));

    v == &clone
}
