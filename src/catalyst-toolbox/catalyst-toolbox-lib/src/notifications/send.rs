use reqwest::{blocking::Client, StatusCode, Url};

use crate::notifications::{
    requests::Request, responses::create_message::CreateMessageResponse, Error,
};

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
