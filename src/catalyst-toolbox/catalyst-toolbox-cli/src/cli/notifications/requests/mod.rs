pub mod create_message;
use serde::Serialize;

#[derive(Serialize)]
pub enum RequestData {
    CreateMessageRequest(create_message::CreateMessage),
}

#[derive(Serialize)]
pub struct Request {
    request: RequestData,
}
