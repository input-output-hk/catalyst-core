use jormungandr_automation::jormungandr::LogLevel;
use std::str::FromStr;
use tracing::level_filters::LevelFilter;
use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::FmtSubscriber;

pub fn init(log_level: LogLevel) -> Result<(), SetGlobalDefaultError> {
    let subscriber = FmtSubscriber::builder()
        .with_file(false)
        .with_target(false)
        .with_max_level(LevelFilter::from_str(log_level.as_ref()).expect("invalid log level"))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}
