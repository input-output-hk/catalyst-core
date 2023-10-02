mod fund;
mod vote_plan;

pub use fund::{CurrentFund, FundDates, FundInfo};
use vit_servicing_station_lib::db::models::funds::FundStageDates;
pub use vote_plan::SingleVotePlanParameters;

pub struct ValidVotePlanParameters {
    pub current_fund: CurrentFund,
    pub next_funds: Vec<FundInfo>,
}

impl From<CurrentFund> for ValidVotePlanParameters {
    fn from(current_fund: CurrentFund) -> Self {
        Self {
            current_fund,
            next_funds: Vec::new(),
        }
    }
}
