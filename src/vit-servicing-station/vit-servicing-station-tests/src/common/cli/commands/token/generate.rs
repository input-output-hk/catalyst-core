use std::process::Command;

pub struct ApiTokenGenerateCommand {
    command: Command,
}

impl ApiTokenGenerateCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn n(mut self, n: u32) -> Self {
        self.command.arg("--n").arg(n.to_string());
        self
    }

    pub fn size(mut self, size: u32) -> Self {
        self.command.arg("--size").arg(size.to_string());
        self
    }

    pub fn build(self) -> Command {
        self.command
    }
}
