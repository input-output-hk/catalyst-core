use crate::cli::notifications::{
    api_params::ApiParams,
    requests::create_message::{CreateMessage, DATETIME_FMT},
    responses::create_message::CreateMessageResponse,
    Error,
};

use chrono::{DateTime, FixedOffset, Local};
use reqwest::blocking::Client;
use reqwest::Url;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Content {
    /// Path to file with notification message, if not provided will be read from the stdin
    content_path: Option<PathBuf>,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct SendNotification {
    #[structopt(flatten)]
    api_params: ApiParams,

    #[structopt(flatten)]
    content_path: Content,

    /// Date and time to send notification of format  "Y-m-d H:M"
    #[structopt(long, parse(try_from_str=parse_date_time))]
    send_date: Option<DateTime<FixedOffset>>,

    /// Ignore user timezones when sending a message
    #[structopt(long, short = "iut")]
    ignore_user_timezones: bool,

    /// Select an specific campaign to send the message to
    #[structopt(long)]
    campaign: Option<String>,

    ///
    #[structopt(long)]
    filter: Option<String>,
}

impl SendNotification {
    pub fn exec(self) -> Result<(), Error> {
        Ok(())
    }
}

fn parse_date_time(dt: &str) -> chrono::ParseResult<DateTime<FixedOffset>> {
    DateTime::parse_from_str(dt, DATETIME_FMT)
}

impl Content {}

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
