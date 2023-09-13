use poem_openapi::NewType;
use serde::Deserialize;

/// The kind of ballot to be cast on this Objective.
/// * `public` - All Ballots are public when cast.
/// * `private` - All Ballots are private.
/// * `cast-private` - All Ballots are cast privately but become public after the tally.
#[derive(NewType, Deserialize)]
pub struct BallotType(String);

impl From<event_db::types::ballot::BallotType> for BallotType {
    fn from(value: event_db::types::ballot::BallotType) -> Self {
        Self(value.0)
    }
}
