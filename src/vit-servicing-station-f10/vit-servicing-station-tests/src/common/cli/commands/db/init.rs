use std::path::Path;
use std::process::Command;
pub struct InitDbCommand {
    command: Command,
}

impl InitDbCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn db_url<P: AsRef<Path>>(mut self, db_url: P) -> Self {
        self.command.arg("--db-url").arg(db_url.as_ref());
        self
    }

    pub fn build(self) -> Command {
        self.command
    }
}
