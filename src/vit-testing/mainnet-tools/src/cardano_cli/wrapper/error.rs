use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum Error {
    #[error("cannot extract slot id")]
    CannotExtractSlotId { regex: String, content: String },
    #[error("io error: {0}")]
    CannotCreateAFile(String),
    #[error("io error: {0}")]
    CannotCreateParentDirectory(String),
    #[error("cannot retrieve parent directory for: '{0:?}'")]
    CannotGetParentDirectory(PathBuf),
    #[error("io error: {0}")]
    CannotWriteAFile(String),
    #[error("io error: {0}")]
    CannotGetOutputFromCommand(String),
    #[error("parse error: {0}")]
    ParseInt(String),
    #[error("cannot parse voter-registration output: {0:?}")]
    Regex(String),
    #[error("cannot parse cardano cli output: {0:?}")]
    CannotParseCardanoCliOutput(String),
    #[error("cannot parse voter-registration output: {0:?}")]
    CannotParseVoterRegistrationOutput(Vec<String>),
    #[error("serialization error: {0}")]
    Json(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization: {0}")]
    Serde(String),
}
