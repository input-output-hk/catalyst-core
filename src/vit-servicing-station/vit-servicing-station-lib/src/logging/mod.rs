use logging_lib::{
    json,
    messages::{DefaultLogMessageBuilder, LogMessageId},
};
use std::time::Duration;

pub fn log_request_elapsed_time(elapsed_time: Duration) {
    let elapsed = elapsed_time.as_nanos().to_string();
    let metadata = json!({ "elapsed_nano_seconds": elapsed });
    DefaultLogMessageBuilder::new()
        .with_level(log::Level::Info)
        .with_tags(vec!["request", "elapsed"])
        .with_message(format!("Request elapsed time: {}ns", elapsed))
        .with_metadata(metadata)
        .with_id(LogMessageId::Other("request_elapsed_time".into()))
        .build()
        .log();
}

pub fn log_rejected_api_key(api_key: String) {
    let metadata = json!({ "api_key": api_key });
    DefaultLogMessageBuilder::new()
        .with_level(log::Level::Info)
        .with_tags(vec!["api_key", "reject"])
        .with_message(format!("Rejected API-Token: {}", api_key))
        .with_metadata(metadata)
        .with_id(LogMessageId::Other("RejectedAPIToken".into()))
        .build()
        .log();
}
