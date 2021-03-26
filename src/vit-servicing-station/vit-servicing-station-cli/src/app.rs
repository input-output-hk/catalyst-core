use crate::api_token::ApiTokenCmd;
use crate::csv::loaders::CsvDataCmd;
use crate::init_db::Db;
use crate::task::ExecTask;
use structopt::StructOpt;

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

    fn exec(&self) -> std::io::Result<Self::ResultValue> {
        match self {
            CliApp::ApiToken(api_token) => api_token.exec(),
            CliApp::CsvData(csv_data) => csv_data.exec(),
            CliApp::Db(db_cmd) => db_cmd.exec(),
        }
    }
}
