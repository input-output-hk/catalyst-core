use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidationError {
    #[error("signature not valid for payload")]
    InvalidSignature,
}
