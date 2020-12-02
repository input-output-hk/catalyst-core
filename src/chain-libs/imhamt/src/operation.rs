use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum InsertError {
    #[error("entry with the provided key already exists")]
    EntryExists,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum RemoveError {
    #[error("could not find the requested key")]
    KeyNotFound,
    #[error("the removed value does not match the expected one")]
    ValueNotMatching,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum UpdateError<T>
where
    T: std::error::Error + std::fmt::Debug + 'static,
{
    #[error("could not find the requested key")]
    KeyNotFound,
    #[error("error while updating the value")]
    ValueCallbackError(#[source] T),
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceError {
    #[error("could not find the requested key")]
    KeyNotFound,
}
