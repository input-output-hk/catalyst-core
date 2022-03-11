use std::time::SystemTime;
use time::{format_description, OffsetDateTime};

pub struct LogEntry {
    timestamp: SystemTime,
    message: String,
}

pub struct Logger {
    entries: Vec<LogEntry>,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn log<S: Into<String>>(&mut self, message: S) {
        let message = message.into();
        let timestamp = SystemTime::now();
        let datetime: OffsetDateTime = timestamp.into();

        println!("{}", self.format_log(message.clone(), datetime));

        self.entries.push(LogEntry { timestamp, message })
    }

    pub fn logs(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|x| {
                let datetime: OffsetDateTime = x.timestamp.into();
                self.format_log(x.message.clone(), datetime)
            })
            .collect()
    }

    pub fn format_log<S: Into<String>>(&self, message: S, datetime: OffsetDateTime) -> String {
        let format = format_description::parse("%d/%m/%Y %T").unwrap();
        format!(
            "[{}] {} ",
            datetime.format(&format).unwrap(),
            message.into()
        )
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
