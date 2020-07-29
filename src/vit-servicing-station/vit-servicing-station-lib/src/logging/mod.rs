use logging_lib::messages::{DefaultLogMessageBuilder, DefaultMetadata, LogMessageId};
use std::time::Duration;

pub fn log_request_elapsed_time(elapsed_time: Duration) {
    let elapsed = elapsed_time.as_nanos().to_string();
    let mut metadata = DefaultMetadata::new();
    metadata.insert("elapsed_nano_seconds".into(), elapsed.clone());
    DefaultLogMessageBuilder::new()
        .with_level(log::Level::Info)
        .with_tags(vec!["request", "elapsed"])
        .with_message(format!("Request elapsed time: {}ns", elapsed))
        .with_id(LogMessageId::Other("request_elapsed_time".into()))
        .build()
        .log();
}

pub fn log_rejected_api_key(api_key: String) {
    let mut metadata = DefaultMetadata::new();
    metadata.insert("api_key".into(), api_key.clone());
    DefaultLogMessageBuilder::new()
        .with_level(log::Level::Info)
        .with_tags(vec!["api_key", "reject"])
        .with_message(format!("Rejected API-Token: {}", api_key))
        .with_id(LogMessageId::Other("RejectedAPIToken".into()))
        .build()
        .log();
}
