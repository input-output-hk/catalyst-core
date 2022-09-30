use chain_crypto::digest::DigestOf;
use color_eyre::eyre::bail;
use color_eyre::Report;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use catalyst_toolbox::rewards::community_advisors::{
    calculate_ca_rewards, ApprovedProposals, CommunityAdvisor, FundSetting, Funds,
    ProposalRewardSlots, ProposalsReviews, Rewards, Seed,
};
use catalyst_toolbox::utils;
use chain_crypto::digest::DigestOf;
use color_eyre::eyre::{bail, eyre};
use color_eyre::Report;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use catalyst_toolbox::community_advisors::models::{
    AdvisorReviewRow, ApprovedProposalRow, ProposalStatus,
};
use catalyst_toolbox::utils::csv::dump_data_to_csv;
use structopt::StructOpt;

#[derive(Debug, Deserialize, StructOpt)]
pub struct FundSettingOpt {
    /// % ratio, range in [0, 100]
    #[structopt(long = "rewards-ratio")]
    proposal_ratio: u8,
    /// % ratio, range in [0, 100]
    #[structopt(long = "bonus-ratio")]
    bonus_ratio: u8,
    /// total amount of funds to be rewarded (integer value)
    #[structopt(long = "funds")]
    total: u64,
}

#[derive(Debug, Deserialize, StructOpt)]
pub struct ProposalRewardsSlotsOpt {
    /// excellent reviews amount of rewards tickets
    #[structopt(long)]
    excellent_slots: u64,
    /// good reviews amount of rewards tickets
    #[structopt(long)]
    good_slots: u64,
    /// maximum number of excellent reviews being rewarded per proposal
    #[structopt(long)]
    max_excellent_reviews: u64,
    /// maximum number of good reviews being rewarded per proposal
    #[structopt(long)]
    max_good_reviews: u64,
}

#[derive(Debug, Deserialize, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct CommunityAdvisors {
    #[structopt(long = "assessments")]
    assessments_path: PathBuf,

    #[structopt(long = "proposals")]
    approved_proposals_path: PathBuf,

    #[structopt(flatten)]
    fund_settings: FundSettingOpt,

    #[structopt(flatten)]
    rewards_slots: ProposalRewardsSlotsOpt,

    #[structopt(long)]
    output: PathBuf,

    #[structopt(long)]
    seed: String,

    /// Output bonus rewards per proposal in a separate file
    #[structopt(long)]
    proposal_bonus_output: Option<PathBuf>,
}

impl CommunityAdvisors {
    pub fn exec(self) -> Result<(), Report> {
        let Self {
            assessments_path,
            approved_proposals_path,
            fund_settings,
            rewards_slots,
            output,
            seed,
            proposal_bonus_output,
        } = self;

        ca_rewards(
            assessments_path,
            approved_proposals_path,
            fund_settings,
            rewards_slots,
            output,
            seed,
            proposal_bonus_output,
        )
    }
}

pub fn ca_rewards(
    assessments_path: PathBuf,
    approved_proposals_path: PathBuf,
    fund_settings: FundSettingOpt,
    rewards_slots: ProposalRewardsSlotsOpt,
    output: PathBuf,
    seed: String,
    proposal_bonus_output: Option<PathBuf>,
) -> Result<(), Report> {
    if fund_settings.bonus_ratio + fund_settings.proposal_ratio != 100 {
        bail!("Wrong ratios: bonus + proposal ratios should be 100");
    }

    let proposal_reviews = read_proposal_reviews(&assessments_path)?;
    let approved_proposals = read_approved_proposals(&approved_proposals_path)?;

    let approved_set = approved_proposals.keys().cloned().collect::<BTreeSet<_>>();
    let proposal_reviews_set = proposal_reviews.keys().cloned().collect::<BTreeSet<_>>();
    let diff = approved_set
        .difference(&proposal_reviews_set)
        .collect::<BTreeSet<_>>();

    if !diff.is_empty() {
        println!(
            "WARNING!, {} proposals without reviews: {:?}",
            diff.len(),
            diff,
        );
    }
    let (good_slots, excellent_slots) = (rewards_slots.good_slots, rewards_slots.excellent_slots);

    let rewards = calculate_ca_rewards(
        proposal_reviews,
        approved_proposals,
        &fund_settings.into(),
        &rewards_slots.into(),
        Seed::from(DigestOf::digest(&seed)),
    );

    let csv_data = rewards_to_csv_data(&rewards.rewards);
    dump_data_to_csv(csv_data.iter(), &output)?;

    println!(
        "Reward for (full) good review {}",
        rewards.base_ticket_reward * Rewards::from(good_slots)
    );
    println!(
        "Reward for (full) excellent review {}",
        rewards.base_ticket_reward * Rewards::from(excellent_slots)
    );
    if let Some(file) = proposal_bonus_output {
        let csv_data = bonus_to_csv_data(rewards.bonus_rewards);
        dump_data_to_csv(csv_data.iter(), &file)?;
    }

    Ok(())
}

fn read_proposal_reviews(path: &Path) -> Result<ProposalsReviews, Report> {
    let reviews: Vec<AdvisorReviewRow> = utils::csv::load_data_from_csv::<_, b','>(path)?;
    let mut proposal_reviews = ProposalsReviews::new();

    for review in reviews.into_iter() {
        proposal_reviews
            .entry(review.proposal_id.clone())
            .or_default()
            .push(review);
    }

    Ok(proposal_reviews)
}

fn read_approved_proposals(path: &Path) -> Result<ApprovedProposals, Report> {
    let approved_proposals: Vec<ApprovedProposalRow> =
        utils::csv::load_data_from_csv::<_, b','>(path)?;
    let proposals = approved_proposals
        .into_iter()
        .filter_map(|proposal| match proposal.status {
            ProposalStatus::Approved => Some(
                Funds::from_str(&proposal.requested_dollars)
                    .map(|funds| (proposal.proposal_id, funds)),
            ),
            ProposalStatus::NotApproved => None,
        })
        .collect::<Result<_, _>>()?;
    Ok(proposals)
}

impl From<FundSettingOpt> for FundSetting {
    fn from(settings: FundSettingOpt) -> Self {
        Self {
            proposal_ratio: settings.proposal_ratio,
            bonus_ratio: settings.bonus_ratio,
            total: Rewards::from(settings.total),
        }
    }
}

impl From<ProposalRewardsSlotsOpt> for ProposalRewardSlots {
    fn from(settings: ProposalRewardsSlotsOpt) -> Self {
        Self {
            excellent_slots: settings.excellent_slots,
            good_slots: settings.good_slots,
            max_good_reviews: settings.max_good_reviews,
            max_excellent_reviews: settings.max_excellent_reviews,
        }
    }
}

fn rewards_to_csv_data(
    rewards: &BTreeMap<CommunityAdvisor, Rewards>,
) -> Result<Vec<impl Serialize>, Report> {
    #[derive(Serialize)]
    struct Entry {
        id: String,
        rewards: u64,
    }

    rewards
        .iter()
        .map(|(id, rewards)| {
            Ok(Entry {
                id: id.clone(),
                rewards: rewards.to_u64().ok_or_else(|| eyre!("Rewards overflow"))?,
            })
        })
        .collect()
}

fn bonus_to_csv_data(rewards: BTreeMap<String, Rewards>) -> Result<Vec<impl Serialize>, Report> {
    #[derive(Serialize)]
    struct Entry {
        proposal_id: String,
        bonus_rewards: u64,
    }

    rewards
        .into_iter()
        .map(|(proposal_id, bonus_rewards)| {
            Ok(Entry {
                proposal_id,
                bonus_rewards: bonus_rewards
                    .to_u64()
                    .ok_or_else(|| eyre!("Rewards overflow"))?,
            })
        })
        .collect()
}
