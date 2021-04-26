pub mod create_message;

#[derive(Serialize)]
pub enum RequestData {
    CreateMessageRequest(create_message::CreateMessage),
}

#[derive(Serialize)]
pub struct Request {
    request: RequestData,
}
