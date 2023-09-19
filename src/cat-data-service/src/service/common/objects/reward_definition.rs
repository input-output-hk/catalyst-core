//! Defines the reward definition.
//!
use super::reward_currency::RewardCurrency;
use poem_openapi::{types::Example, Object};

/// Represents a reward definition.
#[derive(Object)]
#[oai(example = true)]
pub(crate) struct RewardDefiniton {
    /// Currency of the Reward.
    currency: RewardCurrency,

    /// The total value of the reward
    #[oai(validator(minimum(value = "0")))]
    value: i64,
}

impl Example for RewardDefiniton {
    fn example() -> Self {
        Self {
            currency: RewardCurrency::example(),
            value: 100,
        }
    }
}

impl TryFrom<event_db::types::objective::RewardDefinition> for RewardDefiniton {
    type Error = String;
    fn try_from(value: event_db::types::objective::RewardDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            currency: value.currency.try_into()?,
            value: value.value,
        })
    }
}
