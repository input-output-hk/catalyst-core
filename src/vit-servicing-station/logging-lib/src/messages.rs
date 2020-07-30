use serde::Serialize;
use serde_json::Value;

pub type DefaultMetadata = Value;
pub type DefaultLogMessage = LogMessage<DefaultMetadata>;
pub type DefaultLogMessageBuilder = LogMessageBuilder<DefaultMetadata>;

#[derive(Serialize)]
pub enum LogMessageId {
    None,
    Other(String),
}

#[derive(Serialize)]
pub struct LogMessage<Metadata: Serialize> {
    id: LogMessageId,
    level: log::Level,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    message: String,
    timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Metadata>,
}

pub struct LogMessageBuilder<Metadata: Serialize> {
    id: LogMessageId,
    level: log::Level,
    tags: Vec<String>,
    message: Option<String>,
    metadata: Option<Metadata>,
}

impl std::fmt::Display for LogMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl<Metadata: Serialize> LogMessageBuilder<Metadata> {
    pub fn new() -> Self {
        Self {
            id: LogMessageId::None,
            level: log::Level::max(),
            tags: vec![],
            message: None,
            metadata: None,
        }
    }

    pub fn with_id(self, id: LogMessageId) -> Self {
        Self {
            id,
            level: self.level,
            tags: self.tags,
            message: self.message,
            metadata: self.metadata,
        }
    }

    pub fn with_level(self, level: log::Level) -> Self {
        Self {
            id: self.id,
            level,
            tags: self.tags,
            message: self.message,
            metadata: self.metadata,
        }
    }

    pub fn with_tags(self, tags: Vec<&str>) -> Self {
        Self {
            id: self.id,
            level: self.level,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            message: self.message,
            metadata: self.metadata,
        }
    }

    pub fn with_message(self, message: String) -> Self {
        Self {
            id: self.id,
            level: self.level,
            tags: self.tags,
            message: Some(message),
            metadata: self.metadata,
        }
    }

    pub fn with_metadata(self, metadata: Metadata) -> Self {
        Self {
            id: self.id,
            level: self.level,
            tags: self.tags,
            message: self.message,
            metadata: Some(metadata),
        }
    }

    pub fn build(self) -> LogMessage<Metadata> {
        LogMessage {
            id: self.id,
            level: self.level,
            tags: self.tags,
            message: self.message.unwrap_or_default(),
            timestamp: chrono::Utc::now().timestamp(),
            metadata: self.metadata,
        }
    }
}

impl<Metadata: Serialize> Default for LogMessageBuilder<Metadata> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Metadata: Serialize> LogMessage<Metadata> {
    pub fn new(
        id: LogMessageId,
        level: log::Level,
        message: String,
        tags: Vec<String>,
        metadata: Metadata,
    ) -> Self {
        Self {
            id,
            level,
            tags,
            message,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: Some(metadata),
        }
    }

    pub fn log(&self) {
        log::log!(self.level, "{}", serde_json::to_string(self).unwrap())
    }
}

impl<Metadata: Serialize> std::fmt::Display for LogMessage<Metadata> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
