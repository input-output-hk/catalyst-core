use scheduler_service_lib::WrappedPoisonError;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("serialization error")]
    SerializationError(#[from] serde_json::Error),
    #[error("context error")]
    Context(#[from] crate::context::Error),
    #[error(transparent)]
    CardanoCli(#[from] crate::cardano::Error),
    #[error(transparent)]
    FromUtf(#[from] FromUtf8Error),
    #[error(transparent)]
    Scheduler(#[from] scheduler_service_lib::Error),
    #[error(transparent)]
    Poison(#[from] WrappedPoisonError),
}
