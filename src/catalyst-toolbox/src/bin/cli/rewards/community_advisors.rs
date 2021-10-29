use chain_crypto::digest::DigestOf;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::Error;
use catalyst_toolbox::rewards::community_advisors::{
    calculate_ca_rewards, ApprovedProposals, CaRewards, FundSetting, Funds, ProposalRewardSlots,
    ProposalsReviews, Rewards, Seed,
};
use catalyst_toolbox::utils;

use catalyst_toolbox::community_advisors::models::{
    AdvisorReviewRow, ApprovedProposalRow, ProposalStatus,
};
use catalyst_toolbox::utils::csv::dump_data_to_csv;
use structopt::StructOpt;

#[derive(StructOpt)]
struct FundSettingOpt {
    /// % ratio, range in [0, 100]
    #[structopt(long = "rewards-ratio")]
    proposal_ratio: u8,
    /// % ratio, range in [0, 100]
    #[structopt(long = "bonus-ratio")]
    bonus_ratio: u8,
    /// total amount of funds to be rewarded
    #[structopt(long = "funds")]
    total: Funds,
}

#[derive(StructOpt)]
struct ProposalRewardsSlotsOpt {
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

#[derive(StructOpt)]
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
}

impl CommunityAdvisors {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            assessments_path,
            approved_proposals_path,
            fund_settings,
            rewards_slots,
            output,
            seed,
        } = self;

        if fund_settings.bonus_ratio + fund_settings.proposal_ratio != 100 {
            return Err(Error::InvalidInput(
                "Wrong ratios: bonus + proposal ratios should be 100".to_string(),
            ));
        }

        let proposal_reviews = read_proposal_reviews(&assessments_path)?;
        let approved_proposals = read_approved_proposals(&approved_proposals_path)?;

        let rewards = calculate_ca_rewards(
            proposal_reviews,
            &approved_proposals,
            &fund_settings.into(),
            &rewards_slots.into(),
            Seed::from(DigestOf::digest(&seed)),
        );

        let csv_data = rewards_to_csv_data(&rewards);
        dump_data_to_csv(&csv_data, &output)?;
        Ok(())
    }
}

fn read_proposal_reviews(path: &Path) -> Result<ProposalsReviews, Error> {
    let reviews: Vec<AdvisorReviewRow> = utils::csv::load_data_from_csv(path)?;
    let mut proposal_reviews = ProposalsReviews::new();

    for review in reviews.into_iter() {
        proposal_reviews
            .entry(review.proposal_id.clone())
            .or_default()
            .push(review);
    }

    Ok(proposal_reviews)
}

fn read_approved_proposals(path: &Path) -> Result<ApprovedProposals, Error> {
    let approved_proposals: Vec<ApprovedProposalRow> = utils::csv::load_data_from_csv(path)?;
    approved_proposals
        .into_iter()
        .filter_map(|proposal| match proposal.status {
            ProposalStatus::Approved => Some(
                Funds::from_str(&proposal.requested_funds)
                    .map(|funds| (proposal.proposal_id, funds)),
            ),
            ProposalStatus::NotApproved => None,
        })
        .collect::<Result<_, _>>()
        .map_err(|e| Error::InvalidRequestedFunds(e.to_string())) // ParseFixedError does not implement std::Error
}

impl From<FundSettingOpt> for FundSetting {
    fn from(settings: FundSettingOpt) -> Self {
        Self {
            proposal_ratio: settings.proposal_ratio,
            bonus_ratio: settings.bonus_ratio,
            total: settings.total,
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

fn rewards_to_csv_data(rewards: &CaRewards) -> Vec<impl Serialize> {
    #[derive(Serialize)]
    struct Entry {
        id: String,
        rewards: Rewards,
    }

    rewards
        .iter()
        .map(|(id, rewards)| Entry {
            id: id.clone(),
            rewards: *rewards,
        })
        .collect()
}
