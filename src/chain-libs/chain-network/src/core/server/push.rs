use crate::error::{self, Error};
use async_trait::async_trait;
use futures::future::{Future, RemoteHandle};
use futures::task::{Spawn, SpawnError, SpawnExt};

use std::hint::unreachable_unchecked;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

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
    type ResponseFuture: Future<Output = Result<Self::Response, Error>> + Send + Sync;

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
pub struct AsyncAdapter<T, E> {
    handler: T,
    executor: E,
}

impl<T, E> AsyncAdapter<T, E> {
    pub fn new(handler: T, executor: E) -> Self {
        AsyncAdapter { handler, executor }
    }
}

impl<T, E> ResponseHandler for AsyncAdapter<T, E>
where
    T: ResponseHandlerBoxed + 'static,
    T::Response: Send,
    E: Spawn,
{
    type Response = T::Response;
    type ResponseFuture = SpawnResponseFuture<T::Response>;

    fn end_push(self, push_result: Result<(), PushError>) -> Self::ResponseFuture {
        use SpawnResponseFuture::*;

        let fut = self.handler.end_push(push_result);
        match self.executor.spawn_with_handle(fut) {
            Ok(handle) => Spawned(handle),
            Err(e) => Failed(e),
        }
    }
}

pub enum SpawnResponseFuture<R> {
    Spawned(RemoteHandle<Result<R, Error>>),
    Failed(SpawnError),
    Done,
}

impl<R> Future for SpawnResponseFuture<R>
where
    R: Send + 'static,
{
    type Output = Result<R, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<R, Error>> {
        use SpawnResponseFuture::*;

        let this = self.get_mut();
        match this {
            Spawned(handle) => Pin::new(handle).poll(cx),
            Failed(_) => {
                if let Failed(e) = mem::replace(this, Done) {
                    Err(Error::new(error::Code::Internal, e)).into()
                } else {
                    unsafe { unreachable_unchecked() }
                }
            }
            Done => panic!("future polled after completion"),
        }
    }
}
