pub(crate) mod fragments;
pub(crate) mod mockchain;
pub(crate) mod voteplan;

use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Couldn't deserialize entry {entry } in {file}")]
    DeserializeError { file: String, entry: usize },
}
