use crate::error::Error;
use crate::grpc::convert::{error_from_grpc, FromProtobuf};
use futures::prelude::*;
use pin_project::pin_project;
use tonic::Streaming;

use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

#[must_use = "streams do nothing unless polled"]
#[pin_project]
pub struct InboundStream<P, T> {
    #[pin]
    inner: Streaming<P>,
    _phantom: PhantomData<T>,
}

impl<P, T> InboundStream<P, T> {
    pub(crate) fn new(inner: Streaming<P>) -> Self {
        InboundStream {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<P, T> Stream for InboundStream<P, T>
where
    T: FromProtobuf<P>,
{
    type Item = Result<T, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx).map(|opt| {
            opt.map(|item| match item {
                Ok(msg) => {
                    let item = T::from_message(msg)?;
                    Ok(item)
                }
                Err(e) => Err(error_from_grpc(e)),
            })
        })
    }
}
