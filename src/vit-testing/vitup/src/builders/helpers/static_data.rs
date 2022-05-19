use super::time;
use crate::config::Config;
use chain_crypto::bech32::Bech32;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chain_vote::ElectionPublicKey;
use hersir::builder::{Settings, VotePlanSettings};
use vit_servicing_station_tests::common::data::{ValidVotePlanDates, ValidVotePlanParameters};

pub fn build_servicing_station_parameters(
    config: &Config,
    vote_plans: Vec<VotePlanDef>,
    settings: &Settings,
) -> ValidVotePlanParameters {
    let mut parameters = ValidVotePlanParameters::new(
        vote_plans,
        config.data.fund_name.clone(),
        Default::default(),
    );
    parameters.set_voting_power_threshold((config.data.voting_power * 1_000_000) as i64);
    parameters.set_challenges_count(config.data.challenges);
    parameters.set_reviews_count(config.data.reviews);

    let (vote_start_timestamp, tally_start_timestamp, tally_end_timestamp) =
        time::convert_to_human_date(config);

    parameters.dates = ValidVotePlanDates {
        next_fund_start_time: config.data.dates.next_vote_start_time.unix_timestamp(),
        registration_snapshot_time: config.data.dates.snapshot_time.unix_timestamp(),
        next_registration_snapshot_time: config.data.dates.next_snapshot_time.unix_timestamp(),
        insight_sharing_start: config.data.dates.insight_sharing_start.unix_timestamp(),
        proposal_submission_start: config.data.dates.proposal_submission_start.unix_timestamp(),
        refine_proposals_start: config.data.dates.refine_proposals_start.unix_timestamp(),
        finalize_proposals_start: config.data.dates.finalize_proposals_start.unix_timestamp(),
        proposal_assessment_start: config.data.dates.proposal_assessment_start.unix_timestamp(),
        assessment_qa_start: config.data.dates.assessment_qa_start.unix_timestamp(),
        voting_start: vote_start_timestamp.unix_timestamp(),
        voting_tally_end: tally_end_timestamp.unix_timestamp(),
        voting_tally_start: tally_start_timestamp.unix_timestamp(),
    };

    parameters.set_vote_options(config.data.options.clone());
    parameters.set_fund_id(config.data.fund_id);
    parameters.calculate_challenges_total_funds = false;

    if config.vote_plan.private {
        for (alias, data) in settings.vote_plans.iter() {
            if let VotePlanSettings::Private {
                keys,
                vote_plan: _vote_plan,
            } = data
            {
                let key: ElectionPublicKey = keys.election_public_key();
                parameters.set_vote_encryption_key(key.to_bech32_str(), &alias.alias);
            }
        }
    }
    parameters
}
