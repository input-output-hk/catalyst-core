use crate::context::{Context, ContextLock};
use crate::request::Request;
use futures::stream::TryStreamExt;
use futures::FutureExt;
use futures::{channel::mpsc, StreamExt};
use jortestkit::web::api_token::TokenError;
use jortestkit::web::api_token::{APIToken, APITokenManager, API_TOKEN_HEADER};
use std::convert::Infallible;
use std::io::prelude::*;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;
use warp::multipart::FormData;
use warp::Buf;
use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};

impl Reject for crate::context::Error {}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse uuid")]
    CannotParseUuid(#[from] uuid::Error),
    #[error("warp error")]
    WarpError(#[from] warp::Error),
}

impl Reject for Error {}

#[derive(Clone)]
pub struct ServerStopper(mpsc::Sender<()>);

impl ServerStopper {
    pub fn stop(&self) {
        self.0.clone().try_send(()).unwrap();
    }
}

fn job_prameters_binary_body() -> impl Filter<Extract = (Request,), Error = warp::Rejection> + Clone
{
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
    let with_context = warp::any().map(move || context.clone());

    let root = warp::path!("api" / ..).boxed();

    let health = warp::path!("health")
        .and(warp::get())
        .and_then(health_handler)
        .boxed();

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

        root.and(api_token_filter).and(status.or(new)).boxed()
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
    form: FormData,
    context: ContextLock,
) -> Result<impl Reply, Rejection> {
    let request = parse_upload_multipart(form).await.unwrap();
    let mut context_lock = context.lock().unwrap();
    let id = context_lock.new_run(request)?;
    Ok(id).map(|r| warp::reply::json(&r))
}

use bytes::BufMut;
use std::borrow::Borrow;
use warp::multipart::Part;

async fn readall(
    stream: impl futures::Stream<Item = Result<impl Buf, warp::Error>>,
) -> Result<Vec<u8>, warp::Error> {
    stream
        .try_fold(vec![], |mut result, buf| {
            result.put(buf);
            async move { Ok(result) }
        })
        .await
}

async fn parse_upload_multipart(
    form: warp::multipart::FormData,
) -> Result<Request, Box<dyn std::error::Error>> {
    let mut parts: Vec<Part> = form.try_collect().await?;

    let mut get_part = |name: &str| {
        parts
            .iter()
            .position(|part| part.name() == name)
            .map(|p| parts.swap_remove(p))
            .ok_or(format!("{} part not found", name))
    };

    let pin_part = get_part("pin")?;
    let threshold_part = get_part("threshold")?;
    let slot_no_part = get_part("slot-no")?;
    let funds_part = get_part("funds")?;
    let qr_part = get_part("qr")?;

    let qr = readall(qr_part.stream()).await?;
    let pin = String::from_utf8(readall(pin_part.stream()).await?)?;
    let expected_funds: u64 = String::from_utf8(readall(funds_part.stream()).await?)?.parse()?;
    let slot_no: u64 = String::from_utf8(readall(slot_no_part.stream()).await?)?.parse()?;
    let threshold: u64 = String::from_utf8(readall(threshold_part.stream()).await?)?.parse()?;

    Ok(Request {
        qr,
        pin,
        expected_funds,
        slot_no,
        threshold,
    })
}

pub async fn health_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
}

async fn report_invalid(r: Rejection) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::with_status(
        format!("internal error: {:?}", r),
        StatusCode::INTERNAL_SERVER_ERROR,
    ))
}

pub async fn authorize_token(
    token: String,
    context: Arc<std::sync::Mutex<Context>>,
) -> Result<(), Rejection> {
    let api_token = APIToken::from_string(token).map_err(warp::reject::custom)?;

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
