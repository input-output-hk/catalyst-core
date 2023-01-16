use super::api_params::{ApiParams, DEFAULT_PUSHWOOSH_API_URL};
use catalyst_toolbox::notifications::{
    requests::{
        create_message::{
            ContentSettingsBuilder, ContentType, CreateMessage, CreateMessageBuilder, DATETIME_FMT,
        },
        Request, RequestData,
    },
    send::send_create_message,
};
use color_eyre::Report;
use jcli_lib::utils::io;

use clap::Parser;
use reqwest::Url;
use time::OffsetDateTime;

use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Content {
    /// Path to file with notification message, if not provided will be read from the stdin
    content_path: Option<PathBuf>,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Args {
    #[clap(flatten)]
    api_params: ApiParams,

    #[clap(flatten)]
    content_path: Content,

    /// Pushwoosh application code where message will be send
    #[clap(long)]
    application: String,

    /// Date and time to send notification of format  "Y-m-d H:M"
    #[clap(long, value_parser = parse_datetime)]
    send_date: Option<OffsetDateTime>,

    /// Ignore user timezones when sending a message
    #[clap(long)]
    ignore_user_timezones: bool,

    /// Select an specific campaign to send the message to
    #[clap(long)]
    campaign: Option<String>,

    /// Filter options as described by pushwhoosh API
    #[clap(long)]
    filter: Option<String>,

    /// Timezone of send date, for example "America/New_York"
    #[clap(long)]
    timezone: Option<String>,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Json {
    /// Pushwoosh API url
    #[clap(long, default_value = DEFAULT_PUSHWOOSH_API_URL)]
    pub api_url: Url,

    /// Path to file with the json representation of the notification,
    /// if not provided will be read from stdin
    #[clap(flatten)]
    json_path: Content,
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum SendNotification {
    /// Push a notification with setup taken from arguments
    FromArgs(Args),
    /// Push an already built notification from a json object
    FromJson(Json),
}

impl Args {
    pub fn exec(self) -> Result<(), Report> {
        let url = self.api_params.api_url.join("createMessage").unwrap();
        let message = self.build_create_message()?;
        let request = Request::new(RequestData::CreateMessageRequest(message));
        let response = send_create_message(url, &request)?;

        println!("{}", serde_json::to_string_pretty(&response)?);
        Ok(())
    }

    pub fn build_create_message(&self) -> Result<CreateMessage, Report> {
        let content: ContentType = serde_json::from_str(&self.content_path.get_content()?)?;
        let mut content_builder = ContentSettingsBuilder::new()
            .with_timezone(self.timezone.clone())
            .with_campaign(self.campaign.clone())
            .with_filter(self.filter.clone())
            .with_ignore_user_timezones(self.ignore_user_timezones)
            .with_content(content);

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

impl Json {
    pub fn exec(self) -> Result<(), Report> {
        let url = self.api_url.join("createMessage").unwrap();
        let message_data: RequestData = serde_json::from_str(&self.json_path.get_content()?)?;
        let request: Request = Request::new(message_data);
        let response = send_create_message(url, &request)?;

        println!("{}", serde_json::to_string_pretty(&response)?);
        Ok(())
    }
}

impl SendNotification {
    pub fn exec(self) -> Result<(), Report> {
        match self {
            SendNotification::FromArgs(args) => args.exec(),
            SendNotification::FromJson(json) => json.exec(),
        }
    }
}

impl Content {
    pub fn get_content(&self) -> Result<String, Report> {
        let mut reader = io::open_file_read(&self.content_path)?;
        let mut result = String::new();
        reader.read_to_string(&mut result)?;
        Ok(result)
    }
}

fn parse_datetime(dt: &str) -> Result<OffsetDateTime, time::error::Parse> {
    OffsetDateTime::parse(dt, &DATETIME_FMT)
}
