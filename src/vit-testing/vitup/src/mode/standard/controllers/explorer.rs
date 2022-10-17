use jormungandr_automation::jormungandr::{Explorer, ExplorerProcess, NodeAlias, Status};
use std::process::Output;

pub struct ExplorerController {
    alias: NodeAlias,
    explorer_process: ExplorerProcess,
}

impl ExplorerController {
    pub fn new(alias: NodeAlias, explorer_process: ExplorerProcess) -> Self {
        Self {
            alias,
            explorer_process,
        }
    }

    pub(crate) fn status(&self) -> Status {
        if self.explorer_process.is_up() {
            Status::Running
        } else {
            Status::Starting
        }
    }

    pub fn client(&self) -> &Explorer {
        self.explorer_process.client()
    }

    pub(crate) fn address(&self) -> String {
        self.explorer_process
            .configuration()
            .explorer_listen_http_address()
    }
    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn shutdown(self) -> Option<Output> {
        self.explorer_process.shutdown()
    }
}
