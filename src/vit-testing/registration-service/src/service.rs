use crate::context::{ContextLock, State};
use crate::request::Request;
use crate::rest::start_rest_server;
use std::sync::PoisonError;
use thiserror::Error;
use tokio::runtime::{Handle, Runtime};
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct ManagerService {
    context: ContextLock,
    runtime: Option<Runtime>,
}

impl ManagerService {
    pub fn new(context: ContextLock) -> Result<Self, Error> {
        // Do not create a new runtime when already running within a tokio runtime. This is
        // pointless and will result into panic when dropping this structure.
        let runtime = match Handle::try_current() {
            Ok(_) => None,
            Err(_) => Some(Runtime::new()?),
        };

        Ok(Self { context, runtime })
    }

    pub fn spawn(&mut self) -> JoinHandle<()> {
        let server_fut = start_rest_server(self.context.clone());

        // Obtain a handle to the current runtime if present.
        let handle = self
            .runtime
            .as_ref()
            .map(|rt| rt.handle().clone())
            .unwrap_or_else(Handle::current);

        handle.spawn(async move {
            server_fut.await.unwrap();
        })
    }

    pub fn request_to_start(&self) -> Result<Option<(Uuid, Request)>, Error> {
        Ok(match self.context.lock()?.state() {
            State::RequestToStart { job_id, request } => Some((*job_id, request.clone())),
            _ => None,
        })
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
    #[error("cannot stop server")]
    CanntoStopServer,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("cannot acquire lock on context")]
    Poison,
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_err: PoisonError<T>) -> Self {
        Self::Poison
    }
}
