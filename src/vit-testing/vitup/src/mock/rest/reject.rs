use jormungandr_lib::interfaces::FragmentsProcessingSummary;
use std::convert::Infallible;
use warp::{http::StatusCode, Rejection, Reply};

#[derive(Debug)]
pub struct ForcedErrorCode {
    pub code: u16,
}

#[derive(Debug)]
pub struct InvalidBatch {
    pub summary: FragmentsProcessingSummary,
    pub code: u16,
}

#[derive(Debug)]
pub struct GeneralException {
    pub summary: String,
    pub code: u16,
}

impl GeneralException {
    pub fn account_does_not_exist() -> Self {
        Self {
            summary: "".to_string(),
            code: 404,
        }
    }

    pub fn hex_encoding_malformed() -> Self {
        Self {
            summary: "hex encoding malformed".to_string(),
            code: 400,
        }
    }
}

impl warp::reject::Reject for ForcedErrorCode {}
impl warp::reject::Reject for InvalidBatch {}
impl warp::reject::Reject for GeneralException {}

pub async fn report_invalid(r: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(forced_error_code) = r.find::<ForcedErrorCode>() {
        return Ok(warp::reply::with_status(
            "forced rejections".to_string(),
            StatusCode::from_u16(forced_error_code.code).unwrap(),
        ));
    }
    if let Some(invalid_batch) = r.find::<InvalidBatch>() {
        return Ok(warp::reply::with_status(
            serde_json::to_string(&invalid_batch.summary).unwrap(),
            StatusCode::from_u16(invalid_batch.code).unwrap(),
        ));
    }
    if let Some(exception) = r.find::<GeneralException>() {
        return Ok(warp::reply::with_status(
            exception.summary.clone(),
            StatusCode::from_u16(exception.code).unwrap(),
        ));
    }

    Ok(warp::reply::with_status(
        format!("internal error: {:?}", r),
        StatusCode::INTERNAL_SERVER_ERROR,
    ))
}
