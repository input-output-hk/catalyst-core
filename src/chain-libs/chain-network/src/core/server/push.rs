use crate::error::Error;
use async_trait::async_trait;
use futures::future::{BoxFuture, Future};

/// Error detailing the reason of a failure to process
/// a client-streamed request.
#[derive(Debug, thiserror::Error)]
pub enum PushError {
    #[error("request stream error")]
    Inbound(#[source] Error),
    #[error("failed to decode stream data")]
    Decoding(#[source] Error),
    #[error("failed to process request data")]
    Sink(#[source] Error),
}

impl PushError {
    /// Converts the client-streamed request error into the underlying
    /// protocol error, losing the origin information.
    #[inline]
    pub fn flatten(self) -> Error {
        use PushError::*;
        match self {
            Inbound(e) | Decoding(e) | Sink(e) => e,
        }
    }
}

/// An application-defined response hook for client-streamed requests.
///
/// An implementation of this method gives the application a way to
/// observe errors that may occur while receiving and processing
/// items of the request stream, report the termination of the stream,
/// and produce a response for the requesting peer.
pub trait ResponseHandler {
    /// Type of a successful response.
    type Response;

    /// Type of the future that produces the response to the push request.
    type ResponseFuture: Future<Output = Result<Self::Response, Error>> + Send;

    /// Observes the termination result of the request stream and returns
    /// a future used to produce the response.
    fn end_push(self, push_result: Result<(), PushError>) -> Self::ResponseFuture;
}

/// A trait that enables `ResponseHandler` implementations using the
/// `async_trait` macro.
///
/// # Example
///
/// ```
/// use async_trait::async_trait;
///
/// struct MyResponseHandler;
/// struct TokenResponse;
///
/// #[async_trait]
/// impl ResponseHandlerBoxed for MyResponseHandler {
///     type Response = TokenResponse;
///
///     async fn end_push(
///         self,
///         push_result: Result<(), PushError>,
///     ) -> Result<TokenResponse, Error> {
///         println!("push ended with {:?}", push_result);
///         Ok(TokenResponse)
///     }
/// }
/// ```
#[async_trait]
pub trait ResponseHandlerBoxed {
    type Response;

    /// Observes the termination result of the request stream and returns
    /// a future used to produce the response.
    ///
    /// Note that the future object returned by the `end_push` method will be boxed.
    /// To achieve optimal performance, the application developer should consider
    /// implementing the return future of `ResponseHandler::end_push` explicitly
    /// as a static type.
    async fn end_push(self, push_result: Result<(), PushError>) -> Result<Self::Response, Error>;
}

/// Adapter to make `ResponseHandlerBoxed` implementations usable with the
/// `ResponseHandler` bound.
pub struct AsyncAdapter<T>(T);

impl<T> AsyncAdapter<T> {
    pub fn new(inner: T) -> Self {
        AsyncAdapter(inner)
    }
}

impl<T> ResponseHandler for AsyncAdapter<T>
where
    T: ResponseHandlerBoxed + 'static,
{
    type Response = T::Response;
    type ResponseFuture = BoxFuture<'static, Result<T::Response, Error>>;

    fn end_push(self, push_result: Result<(), PushError>) -> Self::ResponseFuture {
        self.0.end_push(push_result)
    }
}
