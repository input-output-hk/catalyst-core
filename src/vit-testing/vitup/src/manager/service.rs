use super::start_rest_server;
use crate::config::VitStartParameters;
use crate::manager::ControlContextLock;
use tokio::runtime::Runtime;

pub struct ManagerService {
    control_context: ControlContextLock,
    runtime: Runtime,
}

impl ManagerService {
    pub fn new(control_context: ControlContextLock) -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            control_context,
        }
    }

    pub fn spawn(&mut self) {
        let server_fut = start_rest_server(self.control_context.clone());

        self.runtime.spawn(async move {
            server_fut.await;
        });
    }

    pub fn request_to_start(&self) -> bool {
        self.control_context.lock().unwrap().request_to_start()
    }

    pub fn request_to_stop(&self) -> bool {
        self.control_context.lock().unwrap().request_to_stop()
    }

    pub fn setup(&self) -> VitStartParameters {
        self.control_context.lock().unwrap().setup().clone()
    }

    pub fn clear_requests(&mut self) {
        self.control_context.lock().unwrap().clear_requests();
    }
}
