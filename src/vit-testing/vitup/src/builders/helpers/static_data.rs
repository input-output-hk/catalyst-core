use super::time;
use crate::config::{Config, Role};
use chain_crypto::bech32::Bech32;
use chain_impl_mockchain::{
    testing::scenario::template::VotePlanDef, tokens::identifier::TokenIdentifier,
};
use chain_vote::ElectionPublicKey;
use hersir::builder::{Settings, VotePlanSettings};
use vit_servicing_station_lib::db::models::{goals::Goal, groups::Group};
use vit_servicing_station_tests::common::data::{
    CurrentFund, FundDates, FundInfo, ValidVotePlanParameters,
};

pub fn build_current_fund(
    config: &Config,
    vote_plans: Vec<VotePlanDef>,
    token_list: Vec<(Role, TokenIdentifier)>,
) -> CurrentFund {
    let template = &config.data.current_fund;

    let (vote_start_timestamp, tally_start_timestamp, tally_end_timestamp) =
        time::convert_to_human_date(config);

    let groups = token_list
        .iter()
        .cloned()
        .map(|(role, token_identifier)| Group {
            fund_id: template.fund_info.fund_id,
            token_identifier: token_identifier.to_string(),
            group_id: role.to_string(),
        })
        .collect();

    let info = FundInfo {
        fund_name: template.fund_info.fund_name.clone(),
        fund_id: template.fund_info.fund_id,
        fund_goal: "".to_string(),
        voting_power_threshold: (template.voting_power * 1_000_000) as i64,
        goals: template
            .fund_info
            .goals
            .iter()
            .cloned()
            .enumerate()
            .map(|(idx, goal_name)| Goal {
                fund_id: template.fund_info.fund_id,
                id: template.fund_info.fund_id * 100 + idx as i32,
                goal_name,
            })
            .collect(),
        results_url: template.fund_info.results_url.clone(),
        survey_url: template.fund_info.survey_url.clone(),
        dates: FundDates {
            next_fund_start_time: template.dates.next_vote_start_time.unix_timestamp(),
            registration_snapshot_time: template.dates.snapshot_time.unix_timestamp(),
            next_registration_snapshot_time: template.dates.next_snapshot_time.unix_timestamp(),
            insight_sharing_start: template.dates.insight_sharing_start.unix_timestamp(),
            proposal_submission_start: template.dates.proposal_submission_start.unix_timestamp(),
            refine_proposals_start: template.dates.refine_proposals_start.unix_timestamp(),
            finalize_proposals_start: template.dates.finalize_proposals_start.unix_timestamp(),
            proposal_assessment_start: template.dates.proposal_assessment_start.unix_timestamp(),
            assessment_qa_start: template.dates.assessment_qa_start.unix_timestamp(),
            voting_start: vote_start_timestamp.unix_timestamp(),
            voting_tally_end: tally_end_timestamp.unix_timestamp(),
            voting_tally_start: tally_start_timestamp.unix_timestamp(),
        },
        groups,
    };

    let mut current_fund = CurrentFund::new(vote_plans, info);

    current_fund.challenges_count = template.challenges;
    current_fund.reviews_count = template.reviews;
    current_fund.vote_options = template.options.clone();
    //TODO: expose as settings in config. Currently business doesn't need it.
    current_fund.calculate_challenges_total_funds = false;

    current_fund
}

pub fn build_next_funds(config: &Config) -> Vec<FundInfo> {
    let current_fund = &config.data.current_fund;

    config
        .data
        .next_funds
        .iter()
        .map(|template| FundInfo {
            fund_name: template.fund_info.fund_name.clone(),
            fund_goal: "".to_string(),
            fund_id: template.fund_info.fund_id,
            voting_power_threshold: (current_fund.voting_power * 1_000_000) as i64,
            goals: template
                .fund_info
                .goals
                .iter()
                .cloned()
                .enumerate()
                .map(|(idx, goal_name)| Goal {
                    fund_id: template.fund_info.fund_id,
                    id: template.fund_info.fund_id * 100 + idx as i32,
                    goal_name,
                })
                .collect(),
            results_url: template.fund_info.results_url.clone(),
            survey_url: template.fund_info.survey_url.clone(),
            dates: FundDates {
                next_fund_start_time: template.dates.next_vote_start_time.unix_timestamp(),
                registration_snapshot_time: template.dates.snapshot_time.unix_timestamp(),
                next_registration_snapshot_time: template.dates.next_snapshot_time.unix_timestamp(),
                insight_sharing_start: template.dates.insight_sharing_start.unix_timestamp(),
                proposal_submission_start: template
                    .dates
                    .proposal_submission_start
                    .unix_timestamp(),
                refine_proposals_start: template.dates.refine_proposals_start.unix_timestamp(),
                finalize_proposals_start: template.dates.finalize_proposals_start.unix_timestamp(),
                proposal_assessment_start: template
                    .dates
                    .proposal_assessment_start
                    .unix_timestamp(),
                assessment_qa_start: template.dates.assessment_qa_start.unix_timestamp(),
                voting_start: template.dates.voting_start.unix_timestamp(),
                voting_tally_end: template.dates.voting_tally_end.unix_timestamp(),
                voting_tally_start: template.dates.voting_tally_start.unix_timestamp(),
            },
            groups: Default::default(),
        })
        .collect()
}

pub fn build_servicing_station_parameters(
    config: &Config,
    token_list: Vec<(Role, TokenIdentifier)>,
    vote_plans: Vec<VotePlanDef>,
    settings: &Settings,
) -> ValidVotePlanParameters {
    let mut parameters =
        ValidVotePlanParameters::from(build_current_fund(config, vote_plans, token_list));
    parameters.next_funds = build_next_funds(config);

    if config.vote_plan.private {
        for (alias, data) in settings.vote_plans.iter() {
            if let VotePlanSettings::Private {
                keys,
                vote_plan: _vote_plan,
            } = data
            {
                let key: ElectionPublicKey = keys.election_key();
                parameters
                    .current_fund
                    .set_vote_encryption_key(key.to_bech32_str(), &alias.alias);
            }
        }
    }
    parameters
}
