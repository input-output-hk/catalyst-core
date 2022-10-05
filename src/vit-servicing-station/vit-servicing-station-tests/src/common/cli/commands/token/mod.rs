mod add;
mod generate;

pub use add::ApiTokenAddCommand;
pub use generate::ApiTokenGenerateCommand;
use std::process::Command;

pub struct ApiTokenCommand {
    command: Command,
}

impl ApiTokenCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn generate(mut self) -> ApiTokenGenerateCommand {
        self.command.arg("generate");
        ApiTokenGenerateCommand::new(self.command)
    }

    pub fn add(mut self) -> ApiTokenAddCommand {
        self.command.arg("add");
        ApiTokenAddCommand::new(self.command)
    }
}
