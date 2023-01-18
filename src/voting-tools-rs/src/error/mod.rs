use nonempty::NonEmpty;
use serde_json::Value;
use thiserror::Error;

use crate::data::{SignedRegistration, VotingPurpose};

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
        /// The expected voting purpose
        expected: VotingPurpose,
        /// The actual voting purpose
        actual: VotingPurpose,
    },

    #[error("the signature, public key, and payload were well-formed, but the signature was not valid for this payload")]
    MismatchedSignature,

    #[error("the list of delegations was empty")]
    EmptyDelegations,
}

/// A registration that failed validation, along with the error
///
/// Useful for providing more detailed error messages about why a particular registration was
/// rejected
///
/// `registration` is an `Option` because some errors prevent us from even generating a
/// [`SignedRegistration`] struct
pub struct InvalidRegistration {
    pub registration: Option<SignedRegistration>,
    pub errors: NonEmpty<RegistrationError>,
}
