//! Main entrypoint to the service
//!
use crate::axum;
use crate::state::State;

use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::try_join;

// These Modules contain endpoints
mod api;
mod docs;

// These modules are utility or generic types/functions
mod generic;
mod service;
mod utilities;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cannot run service, error: {0}")]
    CannotRunService(String),
    #[error(transparent)]
    EventDb(#[from] event_db::error::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Serialize, Debug)]
pub struct ErrorMessage {
    error: String,
}

impl ErrorMessage {
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

/// # Run all web services.
///
/// This will currently run both a Axum based Web API, and a Poem based API.
/// This is only for migration until all endpoints are provided by the Poem service.
///
/// ## Arguments
///
/// `service_addr`: &`SocketAddr` - the address to listen on
/// `metrics_addr`: &`Option<SocketAddr>` - the address to listen on for metrics
/// `state`: `Arc<State>` - the state
///
/// ## Errors
///
/// `Error::CannotRunService` - cannot run the service
/// `Error::EventDbError` - cannot connect to the event db
/// `Error::IoError` - An IO error has occurred.
pub async fn run(
    service_addr: &SocketAddr,
    metrics_addr: &Option<SocketAddr>, // TODO Remove this parameter when Axum is removed.
    state: Arc<State>,
) -> Result<(), Error> {
    // Create service addresses to be used during poem migration.
    // Service address is same as official address but +1 to the port.
    let mut axum_service = *service_addr;
    axum_service.set_port(axum_service.port() + 1);

    try_join!(
        axum::run(&axum_service, metrics_addr, state),
        service::run(service_addr),
    )?;

    Ok(())
}
