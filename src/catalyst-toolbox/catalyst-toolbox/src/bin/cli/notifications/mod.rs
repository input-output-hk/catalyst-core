mod api_params;
mod send;

use clap::Parser;
use color_eyre::Report;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum PushNotifications {
    #[clap(subcommand)]
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
