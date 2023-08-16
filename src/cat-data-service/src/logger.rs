use clap::ValueEnum;
use tracing::{level_filters::LevelFilter, subscriber::SetGlobalDefaultError};
use tracing_subscriber::{
    fmt::{
        format::{DefaultFields, Format},
        FormatEvent,
    },
    FmtSubscriber, Registry,
};

pub const LOG_FORMAT_DEFAULT: &str = "full";
pub const LOG_LEVEL_DEFAULT: &str = "info";

#[derive(ValueEnum, Clone)]
pub enum LogFormat {
    Full,
    Compact,
    Json,
}

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

fn init_impl<T>(format: T, log_level: LogLevel) -> Result<(), SetGlobalDefaultError>
where
    T: FormatEvent<Registry, DefaultFields> + Send + Sync + 'static,
{
    let subscriber = FmtSubscriber::builder()
        .event_format(format)
        .with_max_level(LevelFilter::from_level(log_level.into()))
        .finish();

    tracing::subscriber::set_global_default(subscriber)
}

pub fn init(log_format: LogFormat, log_level: LogLevel) -> Result<(), SetGlobalDefaultError> {
    match log_format {
        LogFormat::Full => init_impl(Format::default(), log_level),
        LogFormat::Compact => init_impl(Format::default().compact(), log_level),
        LogFormat::Json => init_impl(Format::default().json(), log_level),
    }
}
