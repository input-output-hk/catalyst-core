use std::process::Command;

pub struct ApiTokenAddCommand {
    command: Command,
}

impl ApiTokenAddCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn db_url<S: Into<String>>(mut self, db_url: S) -> Self {
        self.command.arg("--db-url").arg(db_url.into());
        self
    }

    pub fn tokens(mut self, tokens: &[String]) -> Self {
        self = self.tokens_as_str(&tokens.join(","));
        self
    }

    pub fn tokens_as_str(mut self, tokens: &str) -> Self {
        self.command.arg("--tokens").arg(tokens);
        self
    }

    pub fn build(self) -> Command {
        self.command
    }
}
