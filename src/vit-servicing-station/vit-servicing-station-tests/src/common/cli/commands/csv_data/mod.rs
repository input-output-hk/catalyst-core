mod load;

pub use load::LoadCsvCommand;
use std::process::Command;

pub struct CsvDataCommand {
    command: Command,
}

impl CsvDataCommand {
    pub fn new(command: Command) -> Self {
        Self { command }
    }

    pub fn load(mut self) -> LoadCsvCommand {
        self.command.arg("load");
        LoadCsvCommand::new(self.command)
    }
}
