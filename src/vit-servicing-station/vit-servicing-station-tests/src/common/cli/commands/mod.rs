mod csv_data;
mod db;
mod token;

use csv_data::CsvDataCommand;
use db::DbCommand;
use token::ApiTokenCommand;

use crate::common::startup::get_cli_exe;
use std::{path::PathBuf, process::Command};

pub struct VitCliCommand {
    exe: PathBuf,
}

impl Default for VitCliCommand {
    fn default() -> Self {
        Self::new(get_cli_exe())
    }
}

impl VitCliCommand {
    pub fn new(exe: PathBuf) -> Self {
        Self { exe }
    }

    pub fn api_token(self) -> ApiTokenCommand {
        let mut command = Command::new(self.exe);
        command.arg("api-token");
        ApiTokenCommand::new(command)
    }

    pub fn db(self) -> DbCommand {
        let mut command = Command::new(self.exe);
        command.arg("db");
        DbCommand::new(command)
    }

    pub fn csv_data(self) -> CsvDataCommand {
        let mut command = Command::new(self.exe);
        command.arg("csv-data");
        CsvDataCommand::new(command)
    }
}
