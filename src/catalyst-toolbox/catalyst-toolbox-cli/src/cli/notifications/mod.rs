mod api_params;
mod send;

use structopt::StructOpt;
use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("error reading file, source: {0}")]
    FileError(#[from] std::io::Error),

    #[error(transparent)]
    NotificationError(#[from] catalyst_toolbox_lib::notifications::Error),
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
