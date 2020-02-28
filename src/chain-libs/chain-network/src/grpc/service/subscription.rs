use super::request_stream::Push;
use super::response_stream::ResponseStream;
use crate::core::server::push::ResponseHandler;
use crate::error::Error;
use crate::grpc::convert::{FromProtobuf, IntoProtobuf};
use futures::prelude::*;
use futures::ready;
use futures::stream::Fuse;
use pin_utils::unsafe_pinned;
use tonic::{Status, Streaming};

use std::pin::Pin;
use std::task::{Context, Poll};

#[must_use = "Subscription objects must be polled"]
pub struct Subscription<St, Pi, Ti, Si, R>
where
    R: ResponseHandler,
{
    inbound: Push<Pi, Ti, Si, R>,
    outbound: Fuse<ResponseStream<St>>,
}

impl<St, Pi, Ti, Si, R> Unpin for Subscription<St, Pi, Ti, Si, R>
where
    St: Unpin,
    Si: Unpin,
    R: ResponseHandler,
    R::ResponseFuture: Unpin,
{
}

impl<St, Pi, Ti, Si, R> Subscription<St, Pi, Ti, Si, R>
where
    R: ResponseHandler,
{
    unsafe_pinned!(inbound: Push<Pi, Ti, Si, R>);
    unsafe_pinned!(outbound: Fuse<ResponseStream<St>>);
}

impl<St, Pi, Ti, Si, R> Subscription<St, Pi, Ti, Si, R>
where
    St: TryStream<Error = Error>,
    St::Ok: IntoProtobuf,
    R: ResponseHandler,
{
    pub(super) fn new(outbound: St, inbound: Streaming<Pi>, sink: Si, response_handler: R) -> Self {
        Subscription {
            inbound: Push::new(inbound, sink, response_handler),
            outbound: ResponseStream::new(outbound).fuse(),
        }
    }
}

impl<St, Pi, Ti, Si, R> Stream for Subscription<St, Pi, Ti, Si, R>
where
    R: ResponseHandler<Response = ()>,
    St: TryStream<Error = Error>,
    St::Ok: IntoProtobuf,
    Ti: FromProtobuf<Pi>,
    Si: Sink<Ti, Error = Error>,
{
    type Item = Result<<St::Ok as IntoProtobuf>::Message, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.as_mut().inbound().poll_step(cx) {
                Poll::Pending => {
                    if self.as_ref().outbound.is_done() {
                        return Poll::Pending;
                    } else {
                        // Will poll outbound below
                    }
                }
                Poll::Ready(None) => {}
                Poll::Ready(Some(Ok(()))) => return None.into(),
                Poll::Ready(Some(Err(status))) => return Some(Err(status)).into(),
            }
            match ready!(self.as_mut().outbound().poll_next(cx)) {
                Some(item) => return Some(item).into(),
                None => {
                    // As per RFC 7540 section 8.1, the stream is
                    // closed after the server ends the response.
                    // Begin flushing the inbound forwarding and closing
                    // its sink.
                    self.as_mut().inbound().begin_closing();
                }
            }
        }
    }
}
