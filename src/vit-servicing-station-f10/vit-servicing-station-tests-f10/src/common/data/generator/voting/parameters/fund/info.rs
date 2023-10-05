use super::dates::FundDates;
use vit_servicing_station_lib_f10::db::models::{funds::Fund, goals::Goal};

#[derive(Debug, Clone)]
pub struct FundInfo {
    pub fund_name: String,
    pub fund_goal: String,
    pub fund_id: i32,
    pub voting_power_threshold: i64,
    pub dates: FundDates,
    pub goals: Vec<Goal>,
    pub results_url: String,
    pub survey_url: String,
}

impl From<FundDates> for FundInfo {
    fn from(dates: FundDates) -> Self {
        FundInfo {
            dates,
            ..Default::default()
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<Fund> for FundInfo {
    fn into(self) -> Fund {
        Fund {
            id: self.fund_id,
            fund_name: self.fund_name,
            fund_goal: self.fund_goal,
            voting_power_threshold: self.voting_power_threshold,
            fund_start_time: self.dates.voting_start,
            fund_end_time: self.dates.voting_tally_start,
            next_fund_start_time: self.dates.next_fund_start_time,
            registration_snapshot_time: self.dates.registration_snapshot_time,
            next_registration_snapshot_time: self.dates.next_registration_snapshot_time,
            chain_vote_plans: vec![],
            challenges: vec![],
            stage_dates: self.dates.into(),
            goals: self.goals,
            results_url: self.results_url,
            survey_url: self.survey_url,
        }
    }
}

impl Default for FundInfo {
    fn default() -> Self {
        Self {
            fund_name: "fund1".to_string(),
            fund_id: 1,
            fund_goal: "".to_string(),
            voting_power_threshold: 500,
            dates: Default::default(),
            goals: vec![Goal {
                id: 1,
                goal_name: "goal1".to_string(),
                fund_id: 1,
            }],
            results_url: "http://localhost/fund/1/results/".to_string(),
            survey_url: "http://localhost/fund/1/survey/".to_string(),
        }
    }
}
