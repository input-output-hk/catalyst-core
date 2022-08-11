mod build;
use std::process::Command;
pub struct AddressCommand {
    command: Command,
}

impl AddressCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn build(mut self) -> build::AddressBuildCommand {
        self.command.arg("build");
        build::AddressBuildCommand::new(self.command)
    }
}
