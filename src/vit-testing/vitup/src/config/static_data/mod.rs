mod current;
mod info;
mod next;

pub use current::CurrentFund;
use info::FundInfo;
pub use next::NextFund;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct StaticData {
    #[serde(default)]
    pub current_fund: CurrentFund,
    #[serde(default)]
    pub next_funds: Vec<NextFund>,
}
