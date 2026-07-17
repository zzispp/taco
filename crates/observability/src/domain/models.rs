use serde_json::Value;
use time::OffsetDateTime;

use super::SystemLogLevel;

#[derive(Clone, Debug, PartialEq)]
pub struct NewSystemLog {
    pub id: String,
    pub occurred_at: OffsetDateTime,
    pub level: SystemLogLevel,
    pub target: String,
    pub message: String,
    pub fields: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemLogSummary {
    pub id: String,
    pub occurred_at: OffsetDateTime,
    pub level: SystemLogLevel,
    pub target: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemLogDetail {
    pub summary: SystemLogSummary,
    pub fields: Value,
}
