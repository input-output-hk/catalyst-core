use std::convert::Infallible;
use thiserror::Error;
use warp::{reply::Response, Rejection, Reply};

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("The data requested data for `{0}` is not available")]
    NotFound(String),

    #[error("Internal error")]
    DatabaseError(#[from] diesel::r2d2::PoolError),

    #[error("Unauthorized token")]
    UnauthorizedToken,

    #[error("Internal error")]
    InternalError(String),
}

impl HandleError {
    fn to_response(&self) -> Response {
        let status_code = match self {
            HandleError::NotFound(_) => warp::http::StatusCode::NOT_FOUND,
            HandleError::DatabaseError(_) => warp::http::StatusCode::SERVICE_UNAVAILABLE,
            HandleError::InternalError(_) => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            HandleError::UnauthorizedToken => warp::http::StatusCode::UNAUTHORIZED,
        };
        warp::reply::with_status(warp::reply(), status_code).into_response()
    }
}

impl warp::Reply for HandleError {
    fn into_response(self) -> Response {
        self.to_response()
    }
}

impl warp::reject::Reject for HandleError {}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(handle_error) = err.find::<HandleError>() {
        return Ok(handle_error.to_response());
    }

    Err(err)
}
