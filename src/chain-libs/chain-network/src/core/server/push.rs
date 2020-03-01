use crate::error::Error;

/// Error detailing the reason of a failure in a client-streamed request.
#[derive(Debug, thiserror::Error)]
pub enum PushError {
    #[error("request stream error")]
    Inbound(#[source] Error),
    #[error("failed to decode stream data")]
    Decoding(#[source] Error),
}

impl PushError {
    /// Converts the client-streamed request error into the underlying
    /// protocol error, losing the origin information.
    #[inline]
    pub fn flatten(self) -> Error {
        use PushError::*;
        match self {
            Inbound(e) | Decoding(e) => e,
        }
    }
}

/// Type alias for inbound stream objects passed to the application.
pub type PushStream<T> = futures::stream::BoxStream<'static, Result<T, PushError>>;
