use crate::state::State;
use axum::{http::StatusCode, Router};
use event_db::queries::snapshot::SnapshotQueries;
use std::{net::SocketAddr, sync::Arc};

mod v0;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cannot run service, error: {0}")]
    CannotRunService(String),
    #[error(transparent)]
    EventDbError(#[from] event_db::Error),
}

// #[tracing::instrument]
pub async fn run_service<EventDB: SnapshotQueries>(
    addr: &SocketAddr,
    state: Arc<State<EventDB>>,
) -> Result<(), Error> {
    tracing::info!("Starting service...");
    tracing::info!("Listening on {}", addr);

    // build our application with a route
    let v0 = v0::v0(state);
    let app = Router::new().nest("/api", v0);

    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| Error::CannotRunService(e.to_string()))?;
    Ok(())
}

async fn handle_result(res: Result<String, Error>) -> (StatusCode, String) {
    match res {
        Ok(res) => (StatusCode::OK, res),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
