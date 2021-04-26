pub mod requests;
mod send;

use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum PushNotifications {
    Send(send::SendNotification),
}

impl PushNotifications {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        use self::PushNotifications::*;
        match self {
            Send(_) => {}
        };
        Ok(())
    }
}
