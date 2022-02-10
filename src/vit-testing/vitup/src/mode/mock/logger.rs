use chrono::offset::Utc;
use chrono::DateTime;
use std::time::SystemTime;

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
        let datetime: DateTime<Utc> = timestamp.into();

        println!("{}", self.format_log(message.clone(), datetime));

        self.entries.push(LogEntry { timestamp, message })
    }

    pub fn logs(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|x| {
                let datetime: DateTime<Utc> = x.timestamp.into();
                self.format_log(x.message.clone(), datetime)
            })
            .collect()
    }

    pub fn format_log<S: Into<String>>(&self, message: S, datetime: DateTime<Utc>) -> String {
        format!("[{}] {} ", datetime.format("%d/%m/%Y %T"), message.into())
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
