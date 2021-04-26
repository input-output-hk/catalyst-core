pub mod common;
pub mod create_message;

pub enum RequestData {
    CreateMessageRequest(create_message::CreateMessage),
}

pub struct Request {
    request: RequestData,
}
