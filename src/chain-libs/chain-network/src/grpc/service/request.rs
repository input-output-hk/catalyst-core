use crate::core::server::PushError;
use crate::grpc::convert::{error_from_grpc, FromProtobuf};
use futures::prelude::*;
use pin_project::pin_project;
use tonic::Streaming;

use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

#[must_use = "streams do nothing unless polled"]
#[pin_project]
pub(super) struct RequestStream<P, T> {
    #[pin]
    inner: Streaming<P>,
    _phantom: PhantomData<T>,
}

impl<P, T> RequestStream<P, T> {
    pub(super) fn new(inner: Streaming<P>) -> Self {
        RequestStream {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<P, T> Stream for RequestStream<P, T>
where
    T: FromProtobuf<P>,
{
    type Item = Result<T, PushError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx).map(|opt| {
            opt.map(|item| match item {
                Ok(msg) => {
                    let item = T::from_message(msg).map_err(PushError::Decoding)?;
                    Ok(item)
                }
                Err(e) => Err(PushError::Inbound(error_from_grpc(e))),
            })
        })
    }
}
