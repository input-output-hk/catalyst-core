use crate::cli::notifications::{
    api_params::ApiParams,
    requests::create_message::{CreateMessage, DATETIME_FMT},
    responses::create_message::CreateMessageResponse,
    Error,
};

use chrono::{DateTime, FixedOffset};
use reqwest::{blocking::Client, StatusCode, Url};
use structopt::StructOpt;

use std::io::Read;
use std::path::PathBuf;

use crate::cli::notifications::requests::create_message::{
    ContentSettingsBuilder, CreateMessageBuilder,
};
use crate::cli::notifications::requests::{Request, RequestData};
use jcli_lib::utils::io;

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

    /// Pushwoosh application code where message will be send
    #[structopt(long)]
    application: String,

    /// Date and time to send notification of format  "Y-m-d H:M"
    #[structopt(long, parse(try_from_str=parse_date_time))]
    send_date: Option<DateTime<FixedOffset>>,

    /// Ignore user timezones when sending a message
    #[structopt(long)]
    ignore_user_timezones: bool,

    /// Select an specific campaign to send the message to
    #[structopt(long)]
    campaign: Option<String>,

    ///
    #[structopt(long)]
    filter: Option<String>,

    /// Timezone of send date, for example "America/New_York"
    #[structopt(long)]
    timezone: Option<String>,
}

impl SendNotification {
    pub fn exec(self) -> Result<(), Error> {
        let url = self.api_params.api_url.join("createMessage").unwrap();
        let message = self.build_create_message()?;
        let request = Request::new(RequestData::CreateMessageRequest(message));
        let response = send_create_message(url, &request)?;

        println!("{}", serde_json::to_string_pretty(&response)?);
        Ok(())
    }

    pub fn build_create_message(&self) -> Result<CreateMessage, Error> {
        let mut content_builder = ContentSettingsBuilder::new()
            .with_timezone(self.timezone.clone())
            .with_campaign(self.campaign.clone())
            .with_filter(self.filter.clone())
            .with_ignore_user_timezones(self.ignore_user_timezones)
            .with_plain_content(self.content_path.get_content()?);
        if let Some(datetime) = self.send_date {
            content_builder = content_builder.with_send_date(datetime);
        }

        CreateMessageBuilder::new()
            .with_auth(self.api_params.access_token.clone())
            .with_application(self.application.clone())
            .add_content_settings(content_builder.build()?)
            .build()
            .map_err(Into::into)
    }
}

fn parse_date_time(dt: &str) -> chrono::ParseResult<DateTime<FixedOffset>> {
    DateTime::parse_from_str(dt, DATETIME_FMT)
}

impl Content {
    pub fn get_content(&self) -> Result<String, Error> {
        let mut reader = io::open_file_read(&self.content_path).map_err(Error::FileError)?;
        let mut result = String::new();
        reader.read_to_string(&mut result)?;
        Ok(result)
    }
}

pub fn send_create_message(
    url: Url,
    notification: &Request,
) -> Result<CreateMessageResponse, Error> {
    let client = Client::new();
    let response = client
        .post(url)
        .body(serde_json::to_string(&notification)?)
        .send()?;
    match response.status() {
        StatusCode::OK => {}
        StatusCode::BAD_REQUEST => {
            return Err(Error::BadDataSent {
                request: serde_json::to_string_pretty(&notification)?,
            })
        }
        _ => {
            return Err(Error::UnsuccessfulRequest {
                response: response.text()?,
            })
        }
    };
    let response_message = response.json()?;
    Ok(response_message)
}
