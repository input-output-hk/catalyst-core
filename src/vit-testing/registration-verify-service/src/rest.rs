use crate::context::ContextLock;
use crate::multipart::parse_multipart;
use futures::FutureExt;
use futures::{channel::mpsc, StreamExt};
use scheduler_service_lib::ServerStopper;
use serde::Serialize;
use snapshot_trigger_service::client::rest::SnapshotRestClient;
use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::multipart::FormData;
use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};

impl Reject for crate::context::Error {}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse uuid")]
    CannotParseUuid(#[from] uuid::Error),
    #[error("cannot retrieve context: {0}")]
    CannotGetContext(String),
    #[error("warp error")]
    WarpError(#[from] warp::Error),
    #[error("cannot find snapshot job id in already cached jobs")]
    NoSnapshotJobIdCached,
    #[error("snapshot error")]
    SnapshotError(#[from] snapshot_trigger_service::client::rest::Error),
    #[error("cannot find job by id: {0}")]
    CannotFindJobByStatus(Uuid),
}

impl Reject for Error {}

pub async fn start_rest_server(context: ContextLock) {
    let (stopper_tx, stopper_rx) = mpsc::channel::<()>(0);
    let stopper_rx = stopper_rx.into_future().map(|_| ());
    context
        .lock()
        .unwrap()
        .set_server_stopper(ServerStopper(stopper_tx));

    let scheduler_context = context.lock().unwrap().into_scheduler_context();
    let shared_scheduler_context = Arc::new(RwLock::new(scheduler_context.clone()));
    let address = *context.lock().unwrap().address();
    let with_context = warp::any().map(move || context.clone());

    let root = warp::path!("api" / ..).boxed();

    let health = scheduler_service_lib::rest::health_filter();

    let job = {
        let root = warp::path!("job" / ..).boxed();

        let new = warp::path!("new")
            .and(warp::post())
            .and(warp::multipart::form().max_length(5_000_000))
            .and(with_context.clone())
            .and_then(job_new_handler)
            .boxed();

        let status = warp::path!("status" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(job_status_handler)
            .boxed();

        let api_token_filter = scheduler_service_lib::rest::token_api_filter(
            shared_scheduler_context.clone(),
            scheduler_context.api_token().is_some(),
        );

        root.and(api_token_filter).and(status.or(new)).boxed()
    };

    let info = {
        let root = warp::path!("info" / ..).boxed();

        let status = warp::path!("snapshot")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(snapshot_info_handler)
            .boxed();

        let api_token_filter = scheduler_service_lib::rest::token_api_filter(
            shared_scheduler_context.clone(),
            scheduler_context.api_token().is_some(),
        );

        root.and(api_token_filter).and(status).boxed()
    };

    let admin = {
        let root = warp::path!("admin" / ..).boxed();

        let update = warp::path!("update" / "job-id" / String)
            .and(warp::post())
            .and(with_context.clone())
            .and_then(update_job_id_handler)
            .boxed();

        let admin_token_filter = scheduler_service_lib::rest::token_admin_filter(
            shared_scheduler_context.clone(),
            scheduler_context.admin_token().is_some(),
        );

        root.and(admin_token_filter).and(update).boxed()
    };

    let api = root
        .and(health.or(job).or(admin).or(info))
        .recover(report_invalid)
        .boxed();

    let server = warp::serve(api);

    let (_, server_fut) = server.bind_with_graceful_shutdown(address, stopper_rx);
    server_fut.await;
}

pub async fn job_status_handler(id: String, context: ContextLock) -> Result<impl Reply, Rejection> {
    let uuid = Uuid::parse_str(&id).map_err(Error::CannotParseUuid)?;
    let context_lock = context
        .lock()
        .map_err(|e| Error::CannotGetContext(e.to_string()))?;
    let status = context_lock.state();
    if status.has_id(&uuid) {
        Err(Error::CannotFindJobByStatus(uuid)).map_err(warp::reject::custom)
    } else {
        Ok(status).map(|r| warp::reply::json(&r))
    }
}

pub async fn update_job_id_handler(
    id: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context
        .lock()
        .map_err(|e| Error::CannotGetContext(e.to_string()))?;
    context_lock.set_snapshot_job_id(id);
    Ok(warp::reply())
}

pub async fn snapshot_info_handler(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context_lock = context
        .lock()
        .map_err(|e| Error::CannotGetContext(e.to_string()))?;
    let config = context_lock.config().clone();

    let snapshot_job_id = config.snapshot_job_id.ok_or(Error::NoSnapshotJobIdCached)?;
    let snapshot_client =
        SnapshotRestClient::new_with_token(config.snapshot_token.clone(), config.snapshot_address);
    let status = snapshot_client
        .get_status(snapshot_job_id)
        .map_err(Error::SnapshotError)?;
    Ok(status).map(|r| warp::reply::json(&r))
}

pub async fn job_new_handler(
    form: FormData,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let request = crate::rest::parse_multipart(form).await?;
    let mut context_lock = context
        .lock()
        .map_err(|e| Error::CannotGetContext(e.to_string()))?;
    let id = context_lock.state_mut().new_run_requested(request)?;
    Ok(id).map(|r| warp::reply::json(&r))
}

async fn report_invalid(r: Rejection) -> Result<impl Reply, Infallible> {
    let mut message = format!("internal error: {:?}", r);
    let mut code = StatusCode::INTERNAL_SERVER_ERROR;

    if let Some(e) = r.find::<crate::multipart::Error>() {
        code = StatusCode::BAD_REQUEST;
        message = e.to_string();
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}
