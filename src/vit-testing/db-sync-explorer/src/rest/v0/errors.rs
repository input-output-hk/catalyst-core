use cardano_serialization_lib::error::DeserializeError;
use thiserror::Error;
use warp::{reply::Response, Rejection, Reply};

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("The data requested data for `{0}` is not available")]
    NotFound(String),

    #[error("Internal error")]
    Connection(#[from] diesel::r2d2::PoolError),

    #[error("Unexpected state of database: {0}")]
    DatabaseInconsistency(String),

    #[error("Unauthorized token")]
    UnauthorizedToken,

    #[error("Deserialize transaction")]
    Deserialize(#[from] DeserializeError),

    #[error("Connection")]
    Database(#[from] diesel::result::Error),

    #[error("Internal error, cause: {0}")]
    InternalError(String),

    #[error("Invalid header {0}, cause: {1}")]
    InvalidHeader(&'static str, &'static str),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Request interrupted")]
    Join(#[from] tokio::task::JoinError),

    #[error("Mock")]
    Mock(#[from] crate::mock::Error),

    #[error("Serialization error")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Template(#[from] mainnet_lib::wallet_state::TemplateError),
}

impl HandleError {
    fn to_status_code(&self) -> warp::http::StatusCode {
        match self {
            HandleError::NotFound(_) => warp::http::StatusCode::NOT_FOUND,
            HandleError::Connection(_) => warp::http::StatusCode::SERVICE_UNAVAILABLE,
            HandleError::InternalError(_) => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            HandleError::UnauthorizedToken => warp::http::StatusCode::UNAUTHORIZED,
            HandleError::InvalidHeader(_, _) => warp::http::StatusCode::BAD_REQUEST,
            HandleError::BadRequest(_) => warp::http::StatusCode::BAD_REQUEST,
            HandleError::DatabaseInconsistency(_) => warp::http::StatusCode::SERVICE_UNAVAILABLE,
            HandleError::Database(_) => warp::http::StatusCode::SERVICE_UNAVAILABLE,
            HandleError::Join(_) => warp::http::StatusCode::SERVICE_UNAVAILABLE,
            HandleError::Mock(_) => warp::http::StatusCode::SERVICE_UNAVAILABLE,
            HandleError::Json(_) => warp::http::StatusCode::SERVICE_UNAVAILABLE,
            HandleError::Deserialize(_) => warp::http::StatusCode::BAD_REQUEST,
            HandleError::Template(_) => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn to_message(&self) -> String {
        format!("{}", self)
    }

    fn to_response(&self) -> Response {
        let status_code = self.to_status_code();
        warp::reply::with_status(warp::reply::json(&self.to_json()), status_code).into_response()
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({"code": self.to_status_code().as_u16(), "message" : self.to_message()})
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
