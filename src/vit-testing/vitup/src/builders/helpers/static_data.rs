use super::time;
use crate::builders::Settings;
use crate::config::VitStartParameters;
use chain_crypto::bech32::Bech32;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use chain_vote::ElectionPublicKey;
use vit_servicing_station_tests::common::data::ValidVotePlanParameters;

pub fn build_servicing_station_parameters(
    fund_name: String,
    input_parameters: &VitStartParameters,
    vote_plans: Vec<VotePlanDef>,
    settings: &Settings,
) -> ValidVotePlanParameters {
    let mut parameters = ValidVotePlanParameters::new(vote_plans, fund_name);
    parameters.set_voting_power_threshold((input_parameters.voting_power * 1_000_000) as i64);
    parameters.set_challenges_count(input_parameters.challenges);
    parameters.set_reviews_count(input_parameters.reviews);

    let (vote_start_timestamp, tally_start_timestamp, tally_end_timestamp) =
        time::convert_to_human_date(
            input_parameters,
            settings
                .network_settings
                .block0
                .blockchain_configuration
                .block0_date,
        );

    parameters.set_voting_start(vote_start_timestamp.timestamp());
    parameters.set_voting_tally_start(tally_start_timestamp.timestamp());
    parameters.set_voting_tally_end(tally_end_timestamp.timestamp());
    parameters.set_next_fund_start_time(input_parameters.next_vote_start_time.timestamp());
    parameters.set_registration_snapshot_time(input_parameters.refresh_time.timestamp());
    parameters.set_fund_id(input_parameters.fund_id);
    parameters.calculate_challenges_total_funds = false;

    if input_parameters.private {
        for (alias, private_key_data) in settings.private_vote_plans.iter() {
            let key: ElectionPublicKey = private_key_data.election_public_key();
            parameters.set_vote_encryption_key(key.to_bech32_str(), alias);
        }
    }
    parameters
}
