use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct NotificationsVerifier {
    auth: String,
}

impl NotificationsVerifier {
    pub fn new<S: Into<String>>(auth: S) -> Self {
        Self { auth: auth.into() }
    }

    pub fn get_message_details<S: Into<String>>(&self, message_id: S) -> MessageDetailsResponse {
        let request = NotificationsRequest::new(&self.auth, &message_id.into());
        println!("{:?}", request);
        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://cp.pushwoosh.com/json/1.3/getMessageDetails")
            .json(&request)
            .send()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "post was unsuccesful");
        let text = response.text().unwrap();
        println!("{:#?}", &text);
        serde_json::from_str(&text).unwrap()
    }

    pub fn verify_message_done_with_text<S: Into<String>>(&self, message_id: S, text: S) {
        let message_id = message_id.into();
        let response = self.get_message_details(&message_id);
        assert_eq!(response.status_code(), 200);
        assert!(response.has_response());

        let message = response.get_message_unsafe();

        assert_eq!(message.code, message_id);
        assert_eq!(message.content.default.as_ref().unwrap(), &text.into());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotificationsRequest {
    request: Request,
}

impl NotificationsRequest {
    pub fn new<S: Into<String>>(auth: S, message: S) -> Self {
        Self {
            request: Request {
                auth: auth.into(),
                message: message.into(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    auth: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageDetailsResponse {
    status_code: u32,
    status_message: String,
    response: Option<Response>,
}

impl MessageDetailsResponse {
    pub fn status_code(&self) -> u32 {
        self.status_code
    }

    pub fn has_response(&self) -> bool {
        self.response.is_some()
    }

    pub fn get_message_unsafe(&self) -> &Message {
        &self.response.as_ref().unwrap().message
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    id: u64,
    created: String,
    send_date: String,
    status: String,
    content: Content,
    platforms: String,
    ignore_user_timezone: String,
    code: String,
    data: Option<String>,
    tracking_code: Option<String>,
    ios_title: Option<String>,
    ios_subtitle: Option<String>,
    ios_root_params: Option<String>,
    android_header: Option<String>,
    android_root_params: Option<String>,
    conditions: Option<String>,
    conditions_operator: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    default: Option<String>,
}
