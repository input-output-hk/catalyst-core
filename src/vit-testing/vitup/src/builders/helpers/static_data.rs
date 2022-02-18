use super::time;
use crate::config::Config;
use chain_crypto::bech32::Bech32;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chain_vote::ElectionPublicKey;
use hersir::builder::{Settings, VotePlanSettings};
use vit_servicing_station_lib::db::models::vote_options::VoteOptions;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;

pub fn build_servicing_station_parameters(
    config: &Config,
    vote_plans: Vec<VotePlanDef>,
    settings: &Settings,
) -> ValidVotePlanParameters {
    let mut parameters = ValidVotePlanParameters::new(vote_plans, config.data.fund_name.clone());
    parameters.set_voting_power_threshold((config.data.voting_power * 1_000_000) as i64);
    parameters.set_challenges_count(config.data.challenges);
    parameters.set_reviews_count(config.data.reviews);

    let (vote_start_timestamp, tally_start_timestamp, tally_end_timestamp) =
        time::convert_to_human_date(config);

    parameters.set_voting_start(vote_start_timestamp.timestamp());
    parameters.set_voting_tally_start(tally_start_timestamp.timestamp());
    parameters.set_voting_tally_end(tally_end_timestamp.timestamp());
    parameters.set_vote_options(VoteOptions::parse_coma_separated_value("yes,no"));
    parameters.set_next_fund_start_time(config.data.next_vote_start_time.timestamp());
    parameters.set_registration_snapshot_time(config.data.snapshot_time.timestamp());
    parameters.set_next_registration_snapshot_time(config.data.next_snapshot_time.timestamp());
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
