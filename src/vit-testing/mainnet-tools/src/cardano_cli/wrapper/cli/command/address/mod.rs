mod build;
use std::process::Command;
pub struct Address {
    command: Command,
}

impl Address {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn build(mut self) -> build::AddressBuilder {
        self.command.arg("build");
        build::AddressBuilder::new(self.command)
    }
}
