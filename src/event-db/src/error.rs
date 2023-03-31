use std::env::VarError;

use bb8::RunError;

/// Event database errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(" Schema in database does not match schema supported by the Crate. The current schema version: {was}, the schema version we expected: {expected}")]
    MismatchedSchema { was: i32, expected: i32 },
    #[error("error: {0}")]
    Unknown(String),
    #[error(transparent)]
    VarError(#[from] VarError),
}

impl From<RunError<tokio_postgres::Error>> for Error {
    fn from(val: RunError<tokio_postgres::Error>) -> Self {
        Self::Unknown(val.to_string())
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(val: tokio_postgres::Error) -> Self {
        Self::Unknown(val.to_string())
    }
}
