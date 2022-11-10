use crate::context::ContextLock;
use crate::request::Request;
use futures::FutureExt;
use futures::{channel::mpsc, StreamExt};
use jortestkit::web::api_token::TokenError;
use scheduler_service_lib::{FileListerError, ServerStopper};
use std::convert::Infallible;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};

impl Reject for crate::context::Error {}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse uuid")]
    CannotParseUuid(#[from] uuid::Error),
    #[error("cannot stop server")]
    CanntoStopServer,
    #[error("cannot acquire lock on context")]
    Poison,
    #[error(transparent)]
    Token(#[from] TokenError),
    #[error(transparent)]
    Send(#[from] futures::channel::mpsc::TrySendError<()>),
    #[error("cannot find job id: {0}")]
    CannotFindJobByStatus(Uuid),
    #[error("no working folder defined")]
    NoWorkingFolder,
}

impl Reject for Error {}
impl Reject for crate::cardano::Error {}

fn job_prameters_json_body() -> impl Filter<Extract = (Request,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

pub async fn start_rest_server(context: ContextLock) -> Result<(), Error> {
    let (stopper_tx, stopper_rx) = mpsc::channel::<()>(0);
    let stopper_rx = stopper_rx.into_future().map(|_| ());
    context
        .lock()
        .unwrap()
        .set_server_stopper(ServerStopper(stopper_tx));

    let is_token_enabled = context.lock().unwrap().api_token().is_some();
    let address = *context.lock().unwrap().address();
    let working_dir = context
        .lock()
        .unwrap()
        .config()
        .working_directory()
        .clone()
        .ok_or(Error::NoWorkingFolder)?;
    let scheduler_context = context.lock().unwrap().into_scheduler_context();
    let shared_scheduler_context = Arc::new(RwLock::new(scheduler_context.clone()));
    let with_context = warp::any().map(move || context.clone());

    let root = warp::path!("api" / ..).boxed();

    let api_token_filter = scheduler_service_lib::rest::token_api_filter(
        shared_scheduler_context.clone(),
        is_token_enabled,
    );
    let files =
        scheduler_service_lib::rest::files_filter(shared_scheduler_context.clone(), working_dir);
    let health = scheduler_service_lib::rest::health_filter();

    let job = {
        let root = warp::path!("job" / ..).boxed();

        let new = warp::path!("new")
            .and(warp::post())
            .and(job_prameters_json_body())
            .and(with_context.clone())
            .and_then(job_new_handler)
            .boxed();

        let status = warp::path!("status" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(status_handler)
            .boxed();

        root.and(api_token_filter.clone())
            .and(files.or(status).or(new))
            .boxed()
    };

    let cardano = {
        let root = warp::path!("cardano" / ..).boxed();

        let tip = warp::path!("network" / "tip")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(network_tip_handler)
            .boxed();

        let transaction = warp::path!("transaction" / "submit")
            .and(warp::post())
            .and(warp::body::content_length_limit(1024 * 16).and(warp::body::bytes()))
            .and(with_context.clone())
            .and_then(submit_transaction_handler)
            .boxed();

        let utxo = warp::path!("utxo" / String)
            .and(warp::get())
            .and(with_context.clone())
            .and_then(query_utxo_handler)
            .boxed();

        root.and(tip.or(transaction).or(utxo)).boxed()
    };

    let api = root
        .and(health.or(job).or(cardano))
        .recover(report_invalid)
        .boxed();

    let server = warp::serve(api);
    let (_, server_fut) = server.bind_with_graceful_shutdown(address, stopper_rx);
    server_fut.await;
    Ok(())
}

pub async fn status_handler(id: String, context: ContextLock) -> Result<impl Reply, Rejection> {
    let uuid = Uuid::parse_str(&id).map_err(Error::CannotParseUuid)?;
    let context_lock = context
        .lock()
        .map_err(|_e| Error::Poison)
        .map_err(warp::reject::custom)?;

    let status = context_lock.state();
    if status.has_id(&uuid) {
        Err(Error::CannotFindJobByStatus(uuid)).map_err(warp::reject::custom)
    } else {
        Ok(status).map(|r| warp::reply::json(&r))
    }
}

pub async fn network_tip_handler(context: ContextLock) -> Result<impl Reply, Rejection> {
    let context_lock = context
        .lock()
        .map_err(|_e| Error::Poison)
        .map_err(warp::reject::custom)?;

    let network = context_lock.config().network;
    Ok(context_lock.cardano_cli_executor().query().tip(network)).map(|r| warp::reply::json(&r))
}

pub async fn query_utxo_handler(
    address: String,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let context_lock = context
        .lock()
        .map_err(|_e| Error::Poison)
        .map_err(warp::reject::custom)?;
    let network = context_lock.config().network;
    Ok(context_lock
        .cardano_cli_executor()
        .query()
        .utxo(address, network))
    .map(|r| warp::reply::json(&r.unwrap().get_total_funds()))
}

pub async fn submit_transaction_handler(
    bytes: warp::hyper::body::Bytes,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let context_lock = context
        .lock()
        .map_err(|_e| Error::Poison)
        .map_err(warp::reject::custom)?;
    let working_dir = context_lock
        .config()
        .working_directory()
        .clone()
        .ok_or(Error::NoWorkingFolder)?;
    context_lock
        .cardano_cli_executor()
        .transaction()
        .submit_from_bytes(bytes.to_vec(), &working_dir)?;
    Ok(warp::reply())
}

pub async fn job_new_handler(
    request: Request,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context
        .lock()
        .map_err(|_e| Error::Poison)
        .map_err(warp::reject::custom)?;
    let id = context_lock.state_mut().new_run_requested(request)?;
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
