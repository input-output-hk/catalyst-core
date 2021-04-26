use crate::cli::notifications::requests::common::Geozone;
use std::collections::HashMap;

pub enum MessageParameters {
    ContentSettings(ContentSettings),
}

pub enum Content {
    Plain(String),
    MultiLanguage(HashMap<String, String>),
}

pub struct ContentSettings {
    send_date: String,
    ignore_user_timezones: bool,
    content: Content,
    timezone: Option<String>,
    campaign: Option<String>,
    geozone: Option<Geozone>,
}

pub struct CreateMessage {
    auth: String,
    application: String,
    notifications: Vec<MessageParameters>,
}
