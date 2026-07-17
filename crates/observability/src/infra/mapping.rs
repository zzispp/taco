use crate::{
    application::{ObservabilityError, ObservabilityResult},
    domain::{SystemLogDetail, SystemLogLevel, SystemLogSummary},
};

use super::records::{SystemLogDetailRecord, SystemLogSummaryRecord};

pub(super) fn summary(record: SystemLogSummaryRecord) -> ObservabilityResult<SystemLogSummary> {
    let level = SystemLogLevel::parse(&record.level).ok_or_else(|| invalid_record(format!("invalid system log level: {}", record.level)))?;
    Ok(SystemLogSummary {
        id: record.id,
        occurred_at: record.occurred_at,
        level,
        target: record.target,
        message: record.message,
    })
}

pub(super) fn detail(record: SystemLogDetailRecord) -> ObservabilityResult<SystemLogDetail> {
    Ok(SystemLogDetail {
        summary: summary(record.summary)?,
        fields: record.fields,
    })
}

pub(super) fn sqlx_error(error: sqlx::Error) -> ObservabilityError {
    match error {
        sqlx::Error::RowNotFound => ObservabilityError::NotFound,
        other => ObservabilityError::Infrastructure(other.to_string()),
    }
}

fn invalid_record(message: String) -> ObservabilityError {
    ObservabilityError::Infrastructure(message)
}
