use crate::api_token::{ApiTokenCmd, Error as ApiTokenError};
use crate::csv::loaders::{CsvDataCmd, Error as CsvDataError};
use crate::init_db::{Db, Error as DbError};
use crate::task::ExecTask;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ApiTokenCmd(#[from] ApiTokenError),
    #[error(transparent)]
    CsvData(#[from] CsvDataError),
    #[error(transparent)]
    Db(#[from] DbError),
}

#[derive(StructOpt)]
pub enum CliApp {
    /// API token related operations
    ApiToken(ApiTokenCmd),
    /// CSV data loaders
    CsvData(CsvDataCmd),
    /// DB related operations
    Db(Db),
}

impl ExecTask for CliApp {
    type ResultValue = ();
    type Error = Error;
    fn exec(&self) -> Result<Self::ResultValue, Error> {
        match self {
            CliApp::ApiToken(api_token) => api_token.exec()?,
            CliApp::CsvData(csv_data) => csv_data.exec()?,
            CliApp::Db(db_cmd) => db_cmd.exec()?,
        };
        Ok(())
    }
}
