use axum::Router;
use std::net::SocketAddr;

mod v0;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cannot run service, error: {0}")]
    CannotRunService(String),
}

#[tracing::instrument]
pub async fn run_service(addr: &SocketAddr) -> Result<(), Error> {
    tracing::info!("Starting service...");
    tracing::info!("Listening on {}", addr);

    // build our application with a route
    let v0 = v0::v0();
    let app = Router::new().nest("/api", v0);

    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| Error::CannotRunService(e.to_string()))?;
    Ok(())
}
