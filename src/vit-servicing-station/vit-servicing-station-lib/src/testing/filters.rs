use serde::Deserialize;
use warp::hyper::{body::Bytes, Response};
/// Extension trait for `Response<Bytes>` to make extracting body easier
pub trait ResponseBytesExt {
    fn as_str(&self) -> &str;

    fn as_json<'a, T>(&'a self) -> T
    where
        T: Deserialize<'a>;
}

impl ResponseBytesExt for Response<Bytes> {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self.body().as_ref()).unwrap()
    }

    fn as_json<'a, T>(&'a self) -> T
    where
        T: Deserialize<'a>,
    {
        let s = self.as_str();
        serde_json::from_str(s).unwrap()
    }
}
