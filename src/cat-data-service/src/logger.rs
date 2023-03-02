use clap::ValueEnum;
use tracing::level_filters::LevelFilter;
use tracing::subscriber::SetGlobalDefaultError;
use tracing_subscriber::FmtSubscriber;

pub const LOG_LEVEL_DEFAULT: &str = "info";

#[derive(ValueEnum, Clone)]
pub enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(val: LogLevel) -> Self {
        match val {
            LogLevel::Info => Self::INFO,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
        }
    }
}

pub fn init(log_level: LogLevel) -> Result<(), SetGlobalDefaultError> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::from_level(log_level.into()))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}
