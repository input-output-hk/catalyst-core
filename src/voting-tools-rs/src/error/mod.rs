use serde_json::Value;
use thiserror::Error;

use crate::data::VotingPurpose;

/// An error encountered during parsing and validation of a Catalyst registration
#[derive(Debug, Error)]
pub enum RegistrationError {
    /// The registration couldn't be parsed from json -> struct
    #[error(
        "the registration metadata couldn't be parsed from JSON into the registration format, json = {}", 
        serde_json::to_string_pretty(&0).unwrap(),
    )]
    RegistrationFormat(Value),

    /// The signature couldn't be parsed from json -> struct
    #[error(
        "the registration metadata couldn't be parsed from JSON into the signature format, json = {}", 
        serde_json::to_string_pretty(&0).unwrap(),
    )]
    SignatureFormat(Value),

    #[error("incorrect voting purpose, expected: {}, actual: {}", expected.0, actual.0)]
    IncorrectVotingPurpose {
        expected: VotingPurpose,
        actual: VotingPurpose,
    },


}
