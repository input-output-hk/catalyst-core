use std::path::Path;
use std::process::Command;
pub struct LoadCsvCommand {
    command: Command,
}

impl LoadCsvCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn db_url<P: AsRef<Path>>(mut self, db_url: P) -> Self {
        self.command.arg("--db-url").arg(db_url.as_ref());
        self
    }

    pub fn funds<P: AsRef<Path>>(mut self, funds: P) -> Self {
        self.command.arg("--funds").arg(funds.as_ref());
        self
    }

    pub fn proposals<P: AsRef<Path>>(mut self, proposals: P) -> Self {
        self.command.arg("--proposals").arg(proposals.as_ref());
        self
    }

    pub fn voteplans<P: AsRef<Path>>(mut self, voteplans: P) -> Self {
        self.command.arg("--voteplans").arg(voteplans.as_ref());
        self
    }

    pub fn challenges<P: AsRef<Path>>(mut self, challenges: P) -> Self {
        self.command.arg("--challenges").arg(challenges.as_ref());
        self
    }

    pub fn advisor_reviews<P: AsRef<Path>>(mut self, reviews: P) -> Self {
        self.command.arg("--reviews").arg(reviews.as_ref());
        self
    }

    pub fn goals<P: AsRef<Path>>(mut self, goals: P) -> Self {
        self.command.arg("--goals").arg(goals.as_ref());
        self
    }

    pub fn build(self) -> Command {
        self.command
    }
}
