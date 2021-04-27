pub mod requests;
pub mod responses;
mod send;

use structopt::StructOpt;
use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("could not build {object_name:?}, missing field {field_name:?}")]
    MissingFieldOnBuilderError {
        object_name: String,
        field_name: String,
    },

    #[error("CreateMessage should contain at least one ContentSettings entry")]
    EmptyContentSettingsError,
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
            Send(_) => {}
        };
        Ok(())
    }
}
