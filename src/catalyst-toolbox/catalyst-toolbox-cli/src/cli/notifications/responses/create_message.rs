use reqwest::StatusCode;
use serde::de::Error;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct InnerResponse {
    #[serde(alias = "Messages")]
    messages: Vec<String>,
}

#[derive(Deserialize)]
pub struct CreateMessageResponse {
    #[serde(deserialize_with = "deserialize_status_code")]
    status_code: StatusCode,
    status_message: String,
    response: InnerResponse,
}

fn deserialize_status_code<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
where
    D: Deserializer<'de>,
{
    StatusCode::from_u16(u16::deserialize(deserializer)?)
        .map_err(|_| D::Error::custom("Invalid StatusCode"))
}

#[cfg(test)]
mod test {
    use super::CreateMessageResponse;
    use reqwest::StatusCode;

    #[test]
    fn test_deserialize() {
        let msg = r#"{
    "status_code": 200,
    "status_message": "OK",
    "response": {
        "Messages": [
            "C3F8-C3863ED4-334AD4F1"
        ]
    }
}"#;
        let message_code = "C3F8-C3863ED4-334AD4F1";
        let response: CreateMessageResponse = serde_json::from_str(msg).expect("valid json data");
        assert_eq!(response.status_code, StatusCode::OK);
        assert_eq!(response.status_message, "OK");
        assert_eq!(response.response.messages.len(), 1);
        assert_eq!(response.response.messages[0], message_code);
    }
}
