use types::http::format_utc_rfc3339_millis;

use crate::{
    application::{ObservabilityError, ObservabilityResult, SystemLogCleanupExecution, SystemLogCleanupExecutionState},
    domain::{SystemLogDetail, SystemLogSummary},
};

use super::dto::{SystemLogCleanupExecutionResponse, SystemLogDetailResponse, SystemLogSummaryResponse};

pub fn summary(value: SystemLogSummary) -> ObservabilityResult<SystemLogSummaryResponse> {
    Ok(SystemLogSummaryResponse {
        log_id: value.id,
        occurred_at: timestamp(value.occurred_at)?,
        level: value.level.code().into(),
        target: value.target,
        message: value.message,
    })
}

pub fn detail(value: SystemLogDetail) -> ObservabilityResult<SystemLogDetailResponse> {
    Ok(SystemLogDetailResponse {
        summary: summary(value.summary)?,
        fields: value.fields,
    })
}

pub fn cleanup_execution(value: SystemLogCleanupExecution) -> SystemLogCleanupExecutionResponse {
    SystemLogCleanupExecutionResponse {
        execution_id: value.execution_id,
        state: cleanup_state(value.state).into(),
        deleted: value.deleted,
        batches: value.batches,
    }
}

fn cleanup_state(value: SystemLogCleanupExecutionState) -> &'static str {
    match value {
        SystemLogCleanupExecutionState::Pending => "pending",
        SystemLogCleanupExecutionState::Running => "running",
        SystemLogCleanupExecutionState::Succeeded => "succeeded",
        SystemLogCleanupExecutionState::Failed => "failed",
        SystemLogCleanupExecutionState::Skipped => "skipped",
        SystemLogCleanupExecutionState::Interrupted => "interrupted",
    }
}

fn timestamp(value: time::OffsetDateTime) -> ObservabilityResult<String> {
    format_utc_rfc3339_millis(value).map_err(|error| ObservabilityError::Infrastructure(error.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::domain::{SystemLogLevel, SystemLogSummary};

    use super::summary;

    #[test]
    fn timestamps_have_fixed_utc_milliseconds() {
        let response = summary(SystemLogSummary {
            id: "log".into(),
            occurred_at: time::OffsetDateTime::UNIX_EPOCH,
            level: SystemLogLevel::Info,
            target: "test".into(),
            message: "message".into(),
        })
        .unwrap();

        assert_eq!(response.occurred_at, "1970-01-01T00:00:00.000Z");
    }
}
