pub mod create_message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestData {
    CreateMessageRequest(create_message::CreateMessage),
}

#[derive(Serialize)]
pub struct Request {
    request: RequestData,
}

impl Request {
    pub fn new(data: RequestData) -> Self {
        Self { request: data }
    }
}
