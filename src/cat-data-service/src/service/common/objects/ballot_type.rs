//! Defines the ballot type.
//!
use poem_openapi::{types::Example, Enum};

/// The kind of ballot to be cast on this Objective.
#[derive(Enum)]
pub(crate) enum BallotType {
    /// All Ballots are public when cast.
    #[oai(rename = "public")]
    Public,

    /// All Ballots are private.
    #[oai(rename = "private")]
    Private,

    /// All Ballots are cast privately but become public after the tally.
    #[oai(rename = "cast-private")]
    CastPrivate,
}

impl Example for BallotType {
    fn example() -> Self {
        Self::Public
    }
}

impl TryFrom<event_db::types::ballot::BallotType> for BallotType {
    type Error = String;
    fn try_from(value: event_db::types::ballot::BallotType) -> Result<Self, Self::Error> {
        match value.0.as_str() {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            "cast-private" => Ok(Self::CastPrivate),
            _ => Err(format!("Unknown ballot type: {}", value.0)),
        }
    }
}
