use crate::config::JobParameters;
use crate::context::ContextLock;
use futures::FutureExt;
use futures::{channel::mpsc, StreamExt};
use jortestkit::web::api_token::TokenError;
use jortestkit::web::api_token::{APIToken, APITokenManager};
use scheduler_service_lib::FileListerError;
use scheduler_service_lib::ServerStopper;
use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};

impl Reject for crate::context::Error {}

impl Reject for Error {}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse uuid")]
    CannotParseUuid(#[from] uuid::Error),
    #[error("cannot parse token")]
    CannotParseToken,
    #[error("cannot find job by uuid: {0}")]
    CannotFindJobByStatus(Uuid),
    #[error("working directory undefined")]
    WorkingDirectoryUndefined,
}

fn job_prameters_json_body(
) -> impl Filter<Extract = (JobParameters,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub async fn start_rest_server(context: ContextLock) {
    let (stopper_tx, stopper_rx) = mpsc::channel::<()>(0);
    let stopper_rx = stopper_rx.into_future().map(|_| ());
    context
        .lock()
        .unwrap()
        .set_server_stopper(ServerStopper(stopper_tx));

    let scheduler_context = context.lock().unwrap().into_scheduler_context();
    let shared_scheduler_context = Arc::new(RwLock::new(scheduler_context.clone()));
    let is_token_enabled = context.lock().unwrap().api_token().is_some();
    let address = *context.lock().unwrap().address();
    let working_dir = context.lock().unwrap().working_directory().to_path_buf();
    let with_context = warp::any().map(move || context.clone());

    let root = warp::path!("api" / ..).boxed();

    let files = scheduler_service_lib::rest::files_filter(
        shared_scheduler_context.clone(),
        working_dir.to_path_buf(),
    );
    let health = scheduler_service_lib::rest::health_filter();

    let job = {
        let root = warp::path!("job" / ..).boxed();

        let new = warp::path!("new")
            .and(warp::post())
            .and(with_context.clone())
            .and(job_prameters_json_body())
            .and_then(job_new_handler)
            .boxed();

        let status = warp::path!("status" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(job_status_handler)
            .boxed();

        let api_token_filter = scheduler_service_lib::rest::token_api_filter(
            shared_scheduler_context.clone(),
            is_token_enabled,
        );

        root.and(api_token_filter)
            .and(files.or(status).or(new))
            .boxed()
    };
    let api = root.and(health.or(job)).recover(report_invalid).boxed();

    let server = warp::serve(api);

    let (_, server_fut) = server.bind_with_graceful_shutdown(address, stopper_rx);
    server_fut.await;
}

pub async fn job_status_handler(id: String, context: ContextLock) -> Result<impl Reply, Rejection> {
    let uuid = Uuid::parse_str(&id).map_err(Error::CannotParseUuid)?;
    let context_lock = context.lock().unwrap();
    let status = context_lock.state();
    if !status.has_id(&uuid) {
        Err(warp::reject::custom(Error::CannotFindJobByStatus(uuid)))
    } else {
        Ok(status).map(|r| warp::reply::json(&r))
    }
}

pub async fn job_new_handler(
    context: ContextLock,
    params: JobParameters,
) -> Result<impl Reply, Rejection> {
    let id = context
        .lock()
        .unwrap()
        .state_mut()
        .new_run_requested(params)?;
    Ok(id).map(|r| warp::reply::json(&r))
}

async fn report_invalid(r: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(e) = r.find::<FileListerError>() {
        Ok(warp::reply::with_status(
            e.to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        Ok(warp::reply::with_status(
            format!("internal error: {:?}", r),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

pub async fn authorize_token(token: String, context: ContextLock) -> Result<(), Rejection> {
    let api_token =
        APIToken::from_string(token).map_err(|_| warp::reject::custom(Error::CannotParseToken))?;

    if context.lock().unwrap().api_token().is_none() {
        return Ok(());
    }

    let manager = APITokenManager::new(context.lock().unwrap().api_token().unwrap())
        .map_err(warp::reject::custom)?;

    if !manager.is_token_valid(api_token) {
        return Err(warp::reject::custom(TokenError::UnauthorizedToken));
    }
    Ok(())
}
