mod api_params;
mod requests;
mod responses;
mod send;

use structopt::StructOpt;
use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    CreateMessageError(#[from] requests::create_message::Error),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error("error reading file, source: {0}")]
    FileError(#[from] std::io::Error),

    #[error("sent data is invalid:\n {request}")]
    BadDataSent { request: String },

    #[error("request was unsuccessful, feedback:\n {response}")]
    UnsuccessfulRequest { response: String },
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum PushNotifications {
    Send(send::SendNotification),
}

impl PushNotifications {
    pub fn exec(self) -> Result<(), Error> {
        use self::PushNotifications::*;
        match self {
            Send(cmd) => cmd.exec()?,
        };
        Ok(())
    }
}
