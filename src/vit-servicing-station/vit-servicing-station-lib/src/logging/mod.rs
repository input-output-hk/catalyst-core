pub mod config;
pub mod messages;
mod methods;

pub use messages::{LogMessage, LogMessageBuilder, LogMessageId};
pub use methods::log;
