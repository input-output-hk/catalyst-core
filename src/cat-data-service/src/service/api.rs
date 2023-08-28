use poem_openapi::{param::Query, payload::PlainText, OpenApi, OpenApiService};
use std::net::SocketAddr;

pub struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
        // API's, parameters and response types should NOT be defined in this file.
        // This should simply call the implementation.
        // No parameter or other processing should be done here.
        match name.0 {
            Some(name) => PlainText(format!("hello, {}!", name)),
            None => PlainText("hello!".to_string()),
        }
    }
}

pub fn api(addr: &SocketAddr) -> OpenApiService<Api, ()> {
    let server_host = format!("http://{}:{}/api", addr.ip(), addr.port());
    OpenApiService::new(Api, "Hello World", "1.0").server(server_host)
}
