use std::marker::PhantomData;

use ::reqwest::{blocking::Response, StatusCode};
use color_eyre::eyre::Result;
use serde::Deserialize;

use self::{rate_limit::RateLimitClient, reqwest::ReqwestClient};

mod rate_limit;
mod reqwest;

pub fn default_http_client(api_key: &str) -> impl HttpClient {
    RateLimitClient::new(ReqwestClient::new(api_key), 10_000)
}

#[cfg(test)]
#[allow(unused)]
fn test_default_client_send_sync() {
    fn check<T: Send + Sync>(_t: T) {}
    check(default_http_client(""));
}

/// Types which can make HTTP requests
pub trait HttpClient:  Send + Sync + 'static {
    fn get<T>(&self, path: &str) -> Result<HttpResponse<T>>
    where
        T: for<'a> Deserialize<'a>;
}

/// A value returned from a HTTP method
pub struct HttpResponse<T: for<'a> Deserialize<'a>> {
    _marker: PhantomData<T>,
    inner: Response,
}

impl<T: for<'a> Deserialize<'a>> HttpResponse<T> {
    pub fn json(self) -> Result<T> {
        Ok(self.inner.json()?)
    }

    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }
}
