use serde::Serialize;

use std::collections::HashMap;

#[derive(Serialize)]
pub enum MessageParameters {
    ContentSettings(ContentSettings),
}

#[derive(Serialize)]
pub enum Content {
    Plain(String),
    MultiLanguage(HashMap<String, String>),
}

#[derive(Serialize)]
pub struct ContentSettings {
    send_date: String,
    content: Content,
    ignore_user_timezones: bool,
    timezone: Option<String>,
    campaign: Option<String>,
    filter: Option<String>,
}

#[derive(Serialize)]
pub struct CreateMessage {
    /// API access token from Pushwoosh Control Panel
    auth: String,
    /// Pushwoosh application code
    application: String,
    /// Push notifications properties
    notifications: Vec<MessageParameters>,
}
