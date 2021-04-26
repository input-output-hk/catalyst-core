use crate::cli::requests;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct SendNotification {}

impl SendNotification {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
