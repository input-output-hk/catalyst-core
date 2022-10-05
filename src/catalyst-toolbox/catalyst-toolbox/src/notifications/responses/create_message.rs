use reqwest::StatusCode;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerResponse {
    #[serde(alias = "Messages")]
    pub messages: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMessageResponse {
    #[serde(
        deserialize_with = "deserialize_status_code",
        serialize_with = "serialize_status_code"
    )]
    pub status_code: StatusCode,
    pub status_message: String,
    pub response: InnerResponse,
}

fn deserialize_status_code<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
where
    D: Deserializer<'de>,
{
    StatusCode::from_u16(u16::deserialize(deserializer)?)
        .map_err(|_| D::Error::custom("Invalid StatusCode"))
}

fn serialize_status_code<S>(status_code: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u16(status_code.as_u16())
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
