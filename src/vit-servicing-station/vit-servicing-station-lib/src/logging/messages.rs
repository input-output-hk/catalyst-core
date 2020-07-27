use serde::export::Formatter;

pub enum LogMessageId {
    None,
    Other(String),
}

pub struct LogMessage {
    id: LogMessageId,
    tags: Vec<String>,
    message: String,
    timestamp: i64,
}

pub struct LogMessageBuilder {
    id: LogMessageId,
    tags: Vec<String>,
    message: Option<String>,
}

impl std::fmt::Display for LogMessageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let id = match self {
            LogMessageId::None => "None",
            LogMessageId::Other(id) => id,
        };
        write!(f, "{}", id)
    }
}

impl LogMessageBuilder {
    pub fn new() -> Self {
        Self {
            id: LogMessageId::None,
            tags: vec![],
            message: None,
        }
    }

    pub fn with_id(self, id: LogMessageId) -> Self {
        Self {
            id,
            tags: self.tags,
            message: self.message,
        }
    }

    pub fn with_tags(self, tags: Vec<String>) -> Self {
        Self {
            id: self.id,
            tags,
            message: self.message,
        }
    }

    pub fn with_message(self, message: String) -> Self {
        Self {
            id: self.id,
            tags: self.tags,
            message: Some(message),
        }
    }

    pub fn build(self) -> LogMessage {
        LogMessage {
            id: self.id,
            tags: self.tags,
            message: self.message.unwrap_or(Default::default()),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

impl LogMessage {
    pub fn new(id: LogMessageId, message: String, tags: Vec<String>) -> Self {
        Self {
            id,
            tags,
            message,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

impl std::fmt::Display for LogMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tags = format!("[{}]", self.tags.join(":"));
        let id = format!("[{}]", self.id);
        write!(
            f,
            "{} - {} - {} - {}",
            id, tags, self.timestamp, self.message
        )
    }
}
