//! Defines the currency of a reward.
//!
use poem_openapi::{types::Example, Enum};

/// Currency of the Reward.
#[derive(Enum)]
pub(crate) enum RewardCurrency {
    #[oai(rename = "ADA")]
    Ada,
}

impl Example for RewardCurrency {
    fn example() -> Self {
        Self::Ada
    }
}

impl TryFrom<String> for RewardCurrency {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "ADA" => Ok(Self::Ada),
            _ => Err(format!("Unknown reward currency: {}", value)),
        }
    }
}
