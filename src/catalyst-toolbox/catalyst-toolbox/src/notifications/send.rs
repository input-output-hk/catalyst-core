use color_eyre::{Report, eyre::bail};
use reqwest::{blocking::Client, StatusCode, Url};

use crate::notifications::{requests::Request, responses::create_message::CreateMessageResponse};

pub fn send_create_message(
    url: Url,
    notification: &Request,
) -> Result<CreateMessageResponse, Report> {
    let client = Client::new();
    let response = client
        .post(url)
        .body(serde_json::to_string(&notification)?)
        .send()?;
    match response.status() {
        StatusCode::OK => {}
        StatusCode::BAD_REQUEST => {
            let request = serde_json::to_string_pretty(&notification)?;
            bail!("bad request: {request:?}")
        }
        _ => {
            let response = response.text()?;
            bail!("unsuccessful request: {response}");
        }
    };
    let response_message = response.json()?;
    Ok(response_message)
}
