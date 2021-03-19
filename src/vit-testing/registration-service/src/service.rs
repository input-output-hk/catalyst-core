use crate::context::{ContextLock, State};
use crate::request::Request;
use crate::rest::start_rest_server;
use tokio::runtime::Runtime;
use uuid::Uuid;

pub struct ManagerService {
    context: ContextLock,
    runtime: Runtime,
}

impl ManagerService {
    pub fn new(context: ContextLock) -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            context,
        }
    }

    pub fn spawn(&mut self) {
        let server_fut = start_rest_server(self.context.clone());

        self.runtime.spawn(async move {
            server_fut.await;
        });
    }

    pub fn request_to_start(&self) -> Option<(Uuid, Request)> {
        match self.context.lock().unwrap().state() {
            State::RequestToStart { job_id, request } => Some((*job_id, request.clone())),
            _ => None,
        }
    }
}
