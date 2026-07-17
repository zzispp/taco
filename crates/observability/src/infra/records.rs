use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow)]
pub(super) struct SystemLogSummaryRecord {
    pub id: String,
    pub occurred_at: OffsetDateTime,
    pub level: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug, FromRow)]
pub(super) struct SystemLogDetailRecord {
    #[sqlx(flatten)]
    pub summary: SystemLogSummaryRecord,
    pub fields: Value,
}
