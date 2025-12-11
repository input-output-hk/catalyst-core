use jormungandr_automation::{
    jormungandr::{JormungandrError, RestError, StartupError},
    testing::block0::Block0Error,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MjolnirError {
    #[error("cannot query rest")]
    RestError(#[from] RestError),
    #[error("cannot bootstrap node")]
    StartupError(#[from] StartupError),
    #[error("jormungandr error")]
    JormungandrError(#[from] JormungandrError),
    #[error("node client error")]
    InternalClientError,
    #[error("pace is too low ({0})")]
    PaceTooLow(u64),
    #[error("get block0 error")]
    Block0Error(#[from] Box<Block0Error>),
}
