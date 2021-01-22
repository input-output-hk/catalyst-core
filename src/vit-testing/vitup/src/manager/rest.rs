use super::file_lister;
use super::State;
use crate::{manager::ControlContextLock, setup::quick::QuickVitBackendParameters};
use futures::FutureExt;
use futures::{channel::mpsc, StreamExt};
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::{Filter, Rejection, Reply};

impl Reject for file_lister::Error {}

impl From<file_lister::Error> for Rejection {
    fn from(other: file_lister::Error) -> Self {
        warp::reject::custom(other)
    }
}

#[derive(Clone)]
pub struct ServerStopper(mpsc::Sender<()>);

impl ServerStopper {
    pub fn stop(&self) {
        self.0.clone().try_send(()).unwrap();
    }
}

pub async fn start_rest_server(context: ControlContextLock) {
    let (stopper_tx, stopper_rx) = mpsc::channel::<()>(0);
    let stopper_rx = stopper_rx.into_future().map(|_| ());
    context
        .lock()
        .unwrap()
        .set_server_stopper(ServerStopper(stopper_tx));

    let working_dir = context.lock().unwrap().working_directory().clone();
    let with_context = warp::any().map(move || context.clone());

    let files = {
        let root = warp::path!("files" / ..).boxed();

        let get = warp::path("get").and(warp::fs::dir(working_dir));
        let list = warp::path!("list")
            .and(warp::get())
            .and(with_context.clone())
            .and_then(file_lister_handler);

        root.and(get.or(list)).boxed()
    };
    let control = {
        let root = warp::path!("control" / ..).boxed();

        let start_default = warp::path!("default")
            .and(with_context.clone())
            .and(warp::post())
            .and_then(start_default_handler);
        let start_custom = warp::path!("custom")
            .and(warp::path::end())
            .and(with_context.clone())
            .and(warp::post())
            .and(warp::body::json())
            .and_then(start_handler);

        let start = warp::path!("start" / ..).and(start_default.or(start_custom));

        let stop = warp::path!("stop")
            .and(warp::post())
            .and(with_context.clone())
            .and_then(stop_handler)
            .boxed();

        root.and(start.or(stop)).boxed()
    };

    let status = warp::path!("status")
        .and(warp::get())
        .and(with_context.clone())
        .and_then(status_handler)
        .boxed();

    let api = files.or(control).or(status).recover(report_invalid).boxed();

    let server = warp::serve(api);
    let (_, server_fut) = server.bind_with_graceful_shutdown(([0, 0, 0, 0], 3030), stopper_rx);
    server_fut.await;
}

pub async fn file_lister_handler(context: ControlContextLock) -> Result<impl Reply, Rejection> {
    let context_lock = context.lock().unwrap();
    Ok(file_lister::dump_json(context_lock.working_directory())?).map(|r| warp::reply::json(&r))
}

pub async fn start_handler(
    context: ControlContextLock,
    parameters: QuickVitBackendParameters,
) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    context_lock.set_parameters(parameters);
    let state = context_lock.state();
    if *state == State::Idle {
        context_lock.start();
        return Ok("start event received".to_owned()).map(|r| warp::reply::json(&r));
    }
    return Ok(format!(
        "Wrong state to stop operation ('{}'), plase wait until state is '{}'",
        state,
        State::Idle
    ))
    .map(|r| warp::reply::json(&r));
}

pub async fn start_default_handler(context: ControlContextLock) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    let state = context_lock.state();
    if *state == State::Idle {
        context_lock.start();
        return Ok("start event received".to_owned()).map(|r| warp::reply::json(&r));
    }
    return Ok(format!(
        "Wrong state to stop operation ('{}'), plase wait until state is '{}'",
        state,
        State::Idle
    ))
    .map(|r| warp::reply::json(&r));
}

pub async fn stop_handler(context: ControlContextLock) -> Result<impl Reply, Rejection> {
    let mut context_lock = context.lock().unwrap();
    let state = context_lock.state();
    if *state == State::Running {
        context_lock.stop();
        return Ok("stop event received".to_owned()).map(|r| warp::reply::json(&r));
    }
    return Ok(format!(
        "Wrong state to stop operation ('{}'), plase wait until state is '{}'",
        state,
        State::Running
    ))
    .map(|r| warp::reply::json(&r));
}

pub async fn status_handler(context: ControlContextLock) -> Result<impl Reply, Rejection> {
    let context_lock = context.lock().unwrap();
    Ok(context_lock.state()).map(|r| warp::reply::json(&r))
}

async fn report_invalid(r: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(e) = r.find::<file_lister::Error>() {
        // It was our specific error type, do whatever we want. We
        // will just print out the error text.
        Ok(warp::reply::with_status(
            e.to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        // Do prettier error reporting for the default error here.
        Ok(warp::reply::with_status(
            format!("internal error: {:?}", r),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
