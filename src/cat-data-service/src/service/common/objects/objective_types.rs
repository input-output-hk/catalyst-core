use poem_openapi::{types::Example, Enum};

/// Objective Type defines the rules of the objective.
#[derive(Enum)]
pub(crate) enum ObjectiveTypes {
    #[oai(rename = "catalyst-simple")]
    Simple,
    #[oai(rename = "catalyst-native")]
    Native,
    #[oai(rename = "catalyst-community-choice")]
    CommunityChoice,
}

impl Example for ObjectiveTypes {
    fn example() -> Self {
        Self::Simple
    }
}

impl TryFrom<String> for ObjectiveTypes {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "catalyst-simple" => Ok(Self::Simple),
            "catalyst-native" => Ok(Self::Native),
            "catalyst-community-choice" => Ok(Self::CommunityChoice),
            _ => Err(format!("Unknown Objective Type: {}", value)),
        }
    }
}
