pub mod community_advisors;
pub mod veterans;
pub mod voters;
pub mod proposers;

use rust_decimal::Decimal;
pub type Funds = Decimal;
// Lets match to the same type as the funds, but naming it funds would be confusing
pub type Rewards = Decimal;
