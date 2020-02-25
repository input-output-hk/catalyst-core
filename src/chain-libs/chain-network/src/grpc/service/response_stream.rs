use crate::error::Error;
use crate::grpc::convert::{error_into_grpc, IntoProtobuf};
use futures::prelude::*;
use futures::ready;
use pin_utils::unsafe_pinned;
use tonic::Status;

use std::pin::Pin;
use std::task::{Context, Poll};

#[must_use = "streams do nothing unless polled"]
pub struct ResponseStream<S> {
    inner: S,
}

impl<S: Unpin> Unpin for ResponseStream<S> {}

impl<S> ResponseStream<S> {
    unsafe_pinned!(inner: S);

    pub(super) fn new(stream: S) -> Self {
        ResponseStream { inner: stream }
    }
}

impl<S> Stream for ResponseStream<S>
where
    S: TryStream<Error = Error>,
    S::Ok: IntoProtobuf,
{
    type Item = Result<<S::Ok as IntoProtobuf>::Message, Status>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let maybe_item = ready!(self.inner().try_poll_next(cx));
        maybe_item
            .map(|item| match item {
                Ok(data) => Ok(data.into_message()),
                Err(e) => Err(error_into_grpc(e)),
            })
            .into()
    }
}
