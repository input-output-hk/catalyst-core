use color_eyre::Result;
use reqwest::StatusCode;

use super::{HttpClient, HttpResponse};

pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Connect,
    Options,
    Trace,
    Path,
}

#[non_exhaustive]
pub struct Spec<'a> {
    pub method: Method,
    pub path: &'a str,
}

pub struct MockClient {
    handler: fn(&Spec) -> (String, StatusCode),
}

impl MockClient {
    pub fn new(handler: fn(&Spec) -> (String, StatusCode)) -> Self {
        Self { handler }
    }
}

impl HttpClient for MockClient {
    fn get<T>(&self, path: &str) -> Result<super::HttpResponse<T>>
    where
        T: for<'a> serde::Deserialize<'a>,
    {
        let spec = Spec {
            method: Method::Get,
            path,
        };

        let (body, status) = (self.handler)(&spec);
        Ok(HttpResponse {
            body,
            status,
            _marker: std::marker::PhantomData,
        })
    }
}

#[test]
fn example_usage() {
    fn function_that_calls_api<T: HttpClient>(client: &T) -> Result<i32> {
        let response = client.get("/example")?;
        response.json()
    }

    let mock_client = MockClient::new(|spec| match spec {
        Spec {
            method: Method::Get,
            path: "/example",
        } => ("123".to_string(), StatusCode::OK),
        _ => ("not found".to_string(), StatusCode::NOT_FOUND),
    });

    let response = function_that_calls_api(&mock_client).unwrap();
    assert_eq!(response, 123);
}
