use super::FundInfo;
use crate::common::data::SingleVotePlanParameters;
use chain_impl_mockchain::testing::scenario::template::VotePlanDef;
use vit_servicing_station_lib::db::models::challenges::Challenge;
use vit_servicing_station_lib::db::models::funds::Fund;
use vit_servicing_station_lib::db::models::vote_options::VoteOptions;

use vit_servicing_station_lib::db::models::voteplans::Voteplan;

pub struct CurrentFund {
    pub vote_plans: Vec<SingleVotePlanParameters>,
    pub vote_options: VoteOptions,
    pub challenges_count: usize,
    pub reviews_count: usize,
    pub calculate_challenges_total_funds: bool,
    pub info: FundInfo,
}

impl CurrentFund {
    pub fn from_single(vote_plan: VotePlanDef, info: FundInfo) -> Self {
        Self::new(vec![vote_plan], info)
    }

    pub fn new(vote_plans: Vec<VotePlanDef>, info: FundInfo) -> Self {
        Self {
            vote_plans: vote_plans.into_iter().map(Into::into).collect(),
            info,
            vote_options: VoteOptions::parse_coma_separated_value("yes,no"),
            challenges_count: 4,
            reviews_count: 1,
            calculate_challenges_total_funds: false,
        }
    }

    pub fn set_vote_encryption_key(&mut self, vote_encryption_key: String, alias: &str) {
        let vote_plan = self
            .vote_plans
            .iter_mut()
            .find(|x| x.alias() == alias)
            .unwrap();
        vote_plan.set_vote_encryption_key(vote_encryption_key);
    }

    pub fn to_fund(&self, vote_plans: Vec<Voteplan>, challenges: Vec<Challenge>) -> Fund {
        Fund {
            id: self.info.fund_id,
            fund_name: self.info.fund_name.clone(),
            fund_goal: self.info.fund_goal.clone(),
            voting_power_threshold: self.info.voting_power_threshold,
            fund_start_time: self.info.dates.voting_start,
            fund_end_time: self.info.dates.voting_tally_start,
            next_fund_start_time: self.info.dates.next_fund_start_time,
            registration_snapshot_time: self.info.dates.registration_snapshot_time,
            next_registration_snapshot_time: self.info.dates.next_registration_snapshot_time,
            chain_vote_plans: vote_plans,
            challenges,
            stage_dates: self.info.dates.clone().into(),
            goals: self.info.goals.clone(),
            results_url: self.info.results_url.clone(),
            survey_url: self.info.survey_url.clone(),
        }
    }
}
