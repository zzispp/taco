use serde_json::{Map, Value};
use time::OffsetDateTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl SystemLogLevel {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    pub const fn priority(self) -> u8 {
        match self {
            Self::Trace => 0,
            Self::Debug => 1,
            Self::Info => 2,
            Self::Warn => 3,
            Self::Error => 4,
        }
    }

    pub const fn from_tracing(value: &tracing::Level) -> Self {
        match *value {
            tracing::Level::TRACE => Self::Trace,
            tracing::Level::DEBUG => Self::Debug,
            tracing::Level::INFO => Self::Info,
            tracing::Level::WARN => Self::Warn,
            tracing::Level::ERROR => Self::Error,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemLogEvent {
    pub id: String,
    pub occurred_at: OffsetDateTime,
    pub level: SystemLogLevel,
    pub target: String,
    pub message: String,
    pub fields: Value,
}

/// Values supplied by a tracing layer before the ingestion runtime assigns an event ID.
pub struct SystemLogEventInput {
    pub occurred_at: OffsetDateTime,
    pub level: SystemLogLevel,
    pub target: String,
    pub message: String,
    pub fields: Map<String, Value>,
}

impl SystemLogEvent {
    pub fn new(input: SystemLogEventInput) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            occurred_at: input.occurred_at,
            level: input.level,
            target: input.target,
            message: input.message,
            fields: Value::Object(input.fields),
        }
    }

    pub(crate) fn serialized_size(&self) -> Result<usize, serde_json::Error> {
        serde_json::to_vec(&serde_json::json!({
            "id": self.id,
            "occurred_at_nanos": self.occurred_at.unix_timestamp_nanos().to_string(),
            "level": self.level.code(),
            "target": self.target,
            "message": self.message,
            "fields": self.fields,
        }))
        .map(|bytes| bytes.len())
    }
}
