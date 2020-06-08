use thiserror::Error;
use warp::reply::Response;

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("The data requested data for `{0}` is not available")]
    NotFound(String),

    #[error("Internal error")]
    DatabaseError(#[from] diesel::r2d2::PoolError),

    #[error("Internal error")]
    InternalError(String),
}

impl warp::Reply for HandleError {
    fn into_response(self) -> Response {
        let status_code = match self {
            HandleError::NotFound(_) => warp::http::StatusCode::NOT_FOUND,
            HandleError::DatabaseError(_) => warp::http::StatusCode::SERVICE_UNAVAILABLE,
            HandleError::InternalError(_) => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        warp::reply::with_status(warp::reply(), status_code).into_response()
    }
}
