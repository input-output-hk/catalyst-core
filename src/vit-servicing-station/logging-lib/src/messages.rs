use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum LogMessageId {
    None,
    Other(String),
}

#[derive(Serialize, Deserialize)]
pub struct LogMessage {
    id: LogMessageId,
    level: log::Level,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    message: String,
    timestamp: i64,
}

pub struct LogMessageBuilder {
    id: LogMessageId,
    level: log::Level,
    tags: Vec<String>,
    message: Option<String>,
}

impl std::fmt::Display for LogMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl LogMessageBuilder {
    pub fn new() -> Self {
        Self {
            id: LogMessageId::None,
            level: log::Level::max(),
            tags: vec![],
            message: None,
        }
    }

    pub fn with_id(self, id: LogMessageId) -> Self {
        Self {
            id,
            level: self.level,
            tags: self.tags,
            message: self.message,
        }
    }

    pub fn with_level(self, level: log::Level) -> Self {
        Self {
            id: self.id,
            level,
            tags: self.tags,
            message: self.message,
        }
    }

    pub fn with_tags(self, tags: Vec<String>) -> Self {
        Self {
            id: self.id,
            level: self.level,
            tags,
            message: self.message,
        }
    }

    pub fn with_message(self, message: String) -> Self {
        Self {
            id: self.id,
            level: self.level,
            tags: self.tags,
            message: Some(message),
        }
    }

    pub fn build(self) -> LogMessage {
        LogMessage {
            id: self.id,
            level: self.level,
            tags: self.tags,
            message: self.message.unwrap_or_default(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

impl Default for LogMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LogMessage {
    pub fn new(id: LogMessageId, level: log::Level, message: String, tags: Vec<String>) -> Self {
        Self {
            id,
            level,
            tags,
            message,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn log(&self) {
        log::log!(self.level, "{}", serde_json::to_string(self).unwrap())
    }
}

impl std::fmt::Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
