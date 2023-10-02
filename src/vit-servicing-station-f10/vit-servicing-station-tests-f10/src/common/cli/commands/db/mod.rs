mod init;

pub use init::InitDbCommand;
use std::process::Command;

pub struct DbCommand {
    command: Command,
}

impl DbCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn init(mut self) -> InitDbCommand {
        self.command.arg("init");
        InitDbCommand::new(self.command)
    }
}
