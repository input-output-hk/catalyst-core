use crate::error::Error;
use crate::grpc::convert::{error_into_grpc, IntoProtobuf};
use futures::prelude::*;
use pin_project::pin_project;
use tonic::Status;

use std::pin::Pin;
use std::task::{Context, Poll};

#[must_use = "streams do nothing unless polled"]
#[pin_project]
pub struct OutboundStream<S> {
    #[pin]
    inner: S,
}

impl<S> OutboundStream<S> {
    pub(crate) fn new(inner: S) -> Self {
        OutboundStream { inner }
    }
}

impl<S> Stream for OutboundStream<S>
where
    S: Stream,
    S::Item: IntoProtobuf,
{
    type Item = <S::Item as IntoProtobuf>::Message;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner
            .poll_next(cx)
            .map(|maybe_item| maybe_item.map(|item| item.into_message()))
    }
}

#[must_use = "streams do nothing unless polled"]
#[pin_project]
pub struct OutboundTryStream<S> {
    #[pin]
    inner: S,
}

impl<S> OutboundTryStream<S> {
    pub(crate) fn new(inner: S) -> Self {
        OutboundTryStream { inner }
    }
}

impl<S> Stream for OutboundTryStream<S>
where
    S: TryStream<Error = Error>,
    S::Ok: IntoProtobuf,
{
    type Item = Result<<S::Ok as IntoProtobuf>::Message, Status>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.try_poll_next(cx).map(|maybe_item| {
            maybe_item.map(|item| match item {
                Ok(data) => Ok(data.into_message()),
                Err(e) => Err(error_into_grpc(e)),
            })
        })
    }
}
