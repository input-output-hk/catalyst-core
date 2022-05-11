use crate::config::JobParameters;
use crate::context::{ContextLock, State};
use crate::rest::start_rest_server;
use tokio::runtime::{Handle, Runtime};
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct ManagerService {
    context: ContextLock,
    runtime: Option<Runtime>,
}

impl ManagerService {
    pub fn new(context: ContextLock) -> Self {
        // Do not create a new runtime when already running within a tokio runtime. This is
        // pointless and will result into panic when dropping this structure.
        let runtime = match Handle::try_current() {
            Ok(_) => None,
            Err(_) => Some(Runtime::new().unwrap()),
        };

        Self { context, runtime }
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
            server_fut.await;
        })
    }

    pub fn request_to_start(&self) -> Option<(Uuid, JobParameters)> {
        match self.context.lock().unwrap().state() {
            State::RequestToStart { job_id, parameters } => Some((*job_id, (*parameters).clone())),
            _ => None,
        }
    }
}
