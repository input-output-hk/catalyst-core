use crate::cli::notifications::requests::create_message::CreateMessage;
use crate::cli::notifications::Error;

use crate::cli::notifications::responses::create_message::CreateMessageResponse;
use reqwest::blocking::Client;
use reqwest::Url;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct SendNotification {}

impl SendNotification {
    pub fn exec(self) -> Result<(), Error> {
        Ok(())
    }
}

pub fn send_create_message(
    url: Url,
    notification: &CreateMessage,
) -> Result<CreateMessageResponse, Error> {
    let mut client = Client::new();
    let response: CreateMessageResponse = client
        .post(url)
        .body(serde_json::to_string(&notification)?)
        .send()?
        .json()?;
    Ok(response)
}
