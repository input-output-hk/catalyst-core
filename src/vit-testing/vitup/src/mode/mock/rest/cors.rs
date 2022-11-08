use reqwest::header::{HeaderMap, HeaderValue};

pub fn default_headers() -> HeaderMap {
    let mut default_headers = HeaderMap::new();
    default_headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    default_headers.insert("vary", HeaderValue::from_static("Origin"));
    default_headers
}

pub fn cors_filter() -> warp::cors::Cors {
    warp::cors()
        .allow_any_origin()
        .allow_methods((vec!["GET", "POST", "OPTIONS", "PUT", "PATCH"]).clone())
        .allow_headers(vec!["content-type"])
        .build()
}
