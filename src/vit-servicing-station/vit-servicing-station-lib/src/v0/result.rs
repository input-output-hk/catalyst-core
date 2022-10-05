use super::errors::HandleError;
use serde::Serialize;
use warp::reply::Response;
use warp::Reply;

pub struct HandlerResult<T>(pub Result<T, HandleError>);

impl<T: Send + Serialize> Reply for HandlerResult<T> {
    fn into_response(self) -> Response {
        match self.0 {
            Ok(res) => warp::reply::json(&res).into_response(),
            Err(error) => error.into_response(),
        }
    }
}
