use poem_openapi::{types::Example, Enum};

/// Currency of the Reward.
#[derive(Enum)]
pub(crate) enum RewardCurrency {
    ADA,
}

impl Example for RewardCurrency {
    fn example() -> Self {
        Self::ADA
    }
}

impl TryFrom<String> for RewardCurrency {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "ADA" => Ok(Self::ADA),
            _ => Err(format!("Unknown Reward Currency: {}", value)),
        }
    }
}
