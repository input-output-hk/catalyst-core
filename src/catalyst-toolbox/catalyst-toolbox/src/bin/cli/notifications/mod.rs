mod api_params;
mod send;

use color_eyre::Report;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum PushNotifications {
    Send(send::SendNotification),
}

impl PushNotifications {
    pub fn exec(self) -> Result<(), Report> {
        use self::PushNotifications::*;
        match self {
            Send(cmd) => cmd.exec()?,
        };
        Ok(())
    }
}
