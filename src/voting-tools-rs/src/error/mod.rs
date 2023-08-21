#![allow(missing_docs)]

use nonempty::NonEmpty;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::data::{NetworkId, SignedRegistration, TxId, VotingPurpose};

/// An error encountered during parsing and validation of a Catalyst registration
#[derive(Debug, Error, PartialEq, Eq, Serialize)]
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

    #[error(
        "the signature, public key, and payload were well-formed, but the signature was not valid for this payload, cbor hash bytes: {}",
        hex::encode(hash_bytes),
    )]
    MismatchedSignature { hash_bytes: [u8; 32] },

    #[error("the list of delegations was empty")]
    EmptyDelegations,

    #[error(
        "stake key has wrong network id, expected {expected}, actual {}",
        actual.map(|id| id.to_string()).unwrap_or_else(|| "None".to_string()),
    )]
    StakeKeyWrongNetwork {
        expected: NetworkId,
        actual: Option<NetworkId>,
    },

    #[error("stake key has wrong type: {0}, expected 14 or 15")]
    StakeKeyWrongType(u8),

    #[error("delegation key error {err}")]
    DelegationError { err: String },

    #[error("stake public key error {err}")]
    StakePublicKeyError { err: String },

    #[error("signature error {err}")]
    SignatureError { err: String },

    #[error("Obsolete registration")]
    ObsoleteRegistration,

    #[error("Cddl parsing failed {err}")]
    CddlParsingFailed { err: String },

    #[error("Cbor deserialization failed {err}")]
    CborDeserializationFailed { err: String },

    #[error("Raw binary conversion of cbor to Registration failure {err}")]
    RawBinCborRegistrationFailure { err: String },

    #[error("Raw binary conversion of cbor to Signature failure {err}")]
    RawBinCborSignatureFailure { err: String },

    #[error("Invalid address prefix {err}")]
    InvalidAddressPrefix { err: String },
}

/// A registration that failed validation, along with the error
///
/// Useful for providing more detailed error messages about why a particular registration was
/// rejected
///
/// `registration` is an `Option` because some errors prevent us from even generating a
/// [`SignedRegistration`] struct
#[derive(Debug, Serialize)]
pub struct InvalidRegistration {
    pub spec_61284: Option<String>,
    pub spec_61285: Option<String>,
    pub registration: Option<SignedRegistration>,
    pub registration_bad_bin: Option<RegistrationCorruptedBin>,
    pub errors: NonEmpty<RegistrationError>,
}

/// Registrations with a corrupted raw cbor binary require extra metadata for context
#[derive(Debug, Serialize)]
pub struct RegistrationCorruptedBin {
    pub tx_id: TxId,
    pub slot: u64,
}
