use crate::api_token::APITokenCmd;
use crate::task::ExecTask;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum CLIApp {
    /// API token related operations
    APIToken(APITokenCmd),
}

impl ExecTask for CLIApp {
    type ResultValue = ();

    fn exec(&self) -> std::io::Result<Self::ResultValue> {
        match self {
            CLIApp::APIToken(api_token) => api_token.exec(),
        }
    }
}
