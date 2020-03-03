use crate::error::Error;

/// Type alias for inbound stream objects passed to the application.
pub type PushStream<T> = futures::stream::BoxStream<'static, Result<T, Error>>;
