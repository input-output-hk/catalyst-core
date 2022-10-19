use crate::config::JobParameters;
use crate::context::{Context, ContextLock};
use futures::FutureExt;
use scheduler_service_lib::{dump_json,FileListerError};
use futures::{channel::mpsc, StreamExt};
use jortestkit::web::api_token::TokenError;
use jortestkit::web::api_token::{APIToken, APITokenManager, API_TOKEN_HEADER};
use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};

impl Reject for crate::context::Error {}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse uuid")]
    CannotParseUuid(#[from] uuid::Error),
    #[error("cannot parse token")]
    CannotParseToken,
}

impl Reject for Error {}

#[derive(Clone)]
pub struct ServerStopper(mpsc::Sender<()>);

impl ServerStopper {
    pub fn stop(&self) {
        self.0.clone().try_send(()).unwrap();
    }
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

    let is_token_enabled = context.lock().unwrap().api_token().is_some();
    let address = *context.lock().unwrap().address();
    let working_dir = context.lock().unwrap().working_directory().clone();
    let with_context = warp::any().map(move || context.clone());

    let root = warp::path!("api" / ..).boxed();

    let files = {
        let root = warp::path!("files" / ..).boxed();

        let get = warp::path("get").and(warp::fs::dir(working_dir));
        let list = warp::path!("list")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(files_handler);

        root.and(get.or(list)).boxed()
    };

    let health = warp::path!("health")
        .and(warp::get())
        .and_then(health_handler)
        .boxed();

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

        let api_token_filter = if is_token_enabled {
            warp::header::header(API_TOKEN_HEADER)
                .and(with_context.clone())
                .and_then(authorize_token)
                .and(warp::any())
                .untuple_one()
                .boxed()
        } else {
            warp::any().boxed()
        };

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
    Ok(context_lock.status_by_id(uuid)).map(|r| warp::reply::json(&r))
}

pub async fn job_new_handler(
    context: ContextLock,
    params: JobParameters,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    let id = context_lock.new_run(params)?;
    Ok(id).map(|r| warp::reply::json(&r))
}

pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
}

pub async fn files_handler(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context_lock = context.lock().unwrap();
    Ok(dump_json(context_lock.working_directory())?).map(|r| warp::reply::json(&r))
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

pub async fn authorize_token(
    token: String,
    context: Arc<std::sync::Mutex<Context>>,
) -> Result<(), Rejection> {
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
