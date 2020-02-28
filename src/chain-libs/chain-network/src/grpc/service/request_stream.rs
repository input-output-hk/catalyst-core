use crate::core::server::push::{PushError, ResponseHandler};
use crate::error::Error;
use crate::grpc::convert::{error_from_grpc, error_into_grpc, FromProtobuf};
use futures::prelude::*;
use futures::ready;
use pin_utils::{unsafe_pinned, unsafe_unpinned};
use tonic::{Status, Streaming};

use std::pin::Pin;
use std::task::{Context, Poll};

#[must_use = "futures do nothing unless polled"]
pub(super) struct Push<P, T, Si, R>
where
    R: ResponseHandler,
{
    state: State<P, T, Si, R>,
}

enum State<P, T, Si, R: ResponseHandler> {
    Forwarding {
        forward: Forward<P, T, Si>,
        response_handler: Option<R>,
    },
    PendingResponse {
        future: R::ResponseFuture,
    },
    Done,
}

struct Forward<P, T, Si> {
    stream: Streaming<P>,
    sink: Si,
    buffered: Option<T>,
    pending_close: bool,
}

impl<P, T, Si, R> Unpin for Push<P, T, Si, R>
where
    Si: Unpin,
    R: ResponseHandler,
    R::ResponseFuture: Unpin,
{
}

impl<P, T, Si> Unpin for Forward<P, T, Si> where Si: Unpin {}

impl<P, T, Si> Forward<P, T, Si> {
    unsafe_pinned!(stream: Streaming<P>); // Streaming is Unpin, but we use the pinned projection for convenience
    unsafe_pinned!(sink: Si);
    unsafe_unpinned!(buffered: Option<T>);
    unsafe_unpinned!(pending_close: bool);
}

impl<P, T, Si, R> Push<P, T, Si, R>
where
    R: ResponseHandler,
{
    pub(super) fn new(stream: Streaming<P>, sink: Si, response_handler: R) -> Self {
        let forward = Forward {
            stream,
            sink,
            buffered: None,
            pending_close: false,
        };
        Push {
            state: State::Forwarding {
                forward,
                response_handler: Some(response_handler),
            },
        }
    }

    pub fn begin_closing(mut self: Pin<&mut Self>) {
        let state = unsafe { &mut self.as_mut().get_unchecked_mut().state };
        match state {
            State::Forwarding {
                forward: Forward { pending_close, .. },
                ..
            } => {
                *pending_close = true;
            }
            State::PendingResponse { .. } | State::Done => {}
        }
    }
}

const POLL_FAILURE: &'static str = "polled `Push` after completion";

impl<P, T, Si, R> Future for Push<P, T, Si, R>
where
    T: FromProtobuf<P>,
    Si: Sink<T, Error = Error>,
    R: ResponseHandler,
{
    type Output = Result<R::Response, Status>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            if let Some(output) = ready!(self.as_mut().poll_step(cx)) {
                return output.into();
            }
        }
    }
}

impl<P, T, Si, R> Push<P, T, Si, R>
where
    T: FromProtobuf<P>,
    Si: Sink<T, Error = Error>,
    R: ResponseHandler,
{
    pub(super) fn poll_step(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<R::Response, Status>>> {
        let state = unsafe { &mut self.as_mut().get_unchecked_mut().state };
        match state {
            State::Forwarding {
                forward,
                response_handler,
            } => {
                let forward = unsafe { Pin::new_unchecked(forward) };
                match ready!(forward.poll_step(cx)) {
                    None => None.into(),
                    Some(push_res) => {
                        let handler = response_handler.take().expect(POLL_FAILURE);
                        let future = handler.end_push(push_res);
                        *state = State::PendingResponse { future };
                        None.into()
                    }
                }
            }
            State::PendingResponse { future } => {
                let future = unsafe { Pin::new_unchecked(future) };
                let res = ready!(future.poll(cx)).map_err(error_into_grpc);
                *state = State::Done;
                Some(res).into()
            }
            State::Done => panic!(POLL_FAILURE),
        }
    }
}

impl<P, T, Si> Forward<P, T, Si>
where
    T: FromProtobuf<P>,
    Si: Sink<T, Error = Error>,
{
    fn poll_step(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(), PushError>>> {
        if let Some(item) = self.as_mut().buffered().take() {
            ready!(self.poll_start_send(cx, item))?;
            None.into()
        } else if self.as_ref().pending_close {
            let res = ready!(self.sink().poll_close(cx)).map_err(PushError::Sink);
            Some(res).into()
        } else {
            match self.as_mut().stream().poll_next(cx) {
                Poll::Pending => {
                    ready!(self.sink().poll_flush(cx)).map_err(PushError::Sink)?;
                    Poll::Pending
                }
                Poll::Ready(Some(Ok(msg))) => {
                    let item = T::from_message(msg).map_err(PushError::Decoding)?;
                    ready!(self.poll_start_send(cx, item))?;
                    None.into()
                }
                Poll::Ready(None) => {
                    *self.pending_close() = true;
                    None.into()
                }
                Poll::Ready(Some(Err(status))) => {
                    let err = error_from_grpc(status);
                    Some(Err(PushError::Inbound(err))).into()
                }
            }
        }
    }

    fn poll_start_send(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        item: T,
    ) -> Poll<Result<(), PushError>> {
        let mut sink = self.as_mut().sink();
        match sink.as_mut().poll_ready(cx) {
            Poll::Ready(Ok(())) => sink.start_send(item).map_err(PushError::Sink).into(),
            Poll::Pending => {
                *self.buffered() = Some(item);
                Poll::Pending
            }
            Poll::Ready(Err(e)) => Err(PushError::Sink(e)).into(),
        }
    }
}
