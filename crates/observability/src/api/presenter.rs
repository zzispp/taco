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
        detail_schema_version: value.detail_schema_version,
        rows_deleted: value.rows_deleted,
        dropped_partitions: value.dropped_partitions,
        legacy_total_deleted: value.legacy_total_deleted,
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
    use crate::{
        application::{SystemLogCleanupExecution, SystemLogCleanupExecutionState},
        domain::{SystemLogLevel, SystemLogSummary},
    };

    use super::{cleanup_execution, summary};

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

    #[test]
    fn cleanup_execution_preserves_current_metric_units() {
        let response = cleanup_execution(SystemLogCleanupExecution {
            execution_id: "execution".into(),
            state: SystemLogCleanupExecutionState::Succeeded,
            detail_schema_version: Some(2),
            rows_deleted: Some(5),
            dropped_partitions: Some(2),
            legacy_total_deleted: None,
            batches: Some(3),
        });

        assert_eq!(response.detail_schema_version, Some(2));
        assert_eq!(response.rows_deleted, Some(5));
        assert_eq!(response.dropped_partitions, Some(2));
        assert_eq!(response.legacy_total_deleted, None);
        assert_eq!(response.batches, Some(3));
    }

    #[test]
    fn cleanup_execution_keeps_legacy_total_separate_from_current_metrics() {
        let response = cleanup_execution(SystemLogCleanupExecution {
            execution_id: "execution".into(),
            state: SystemLogCleanupExecutionState::Succeeded,
            detail_schema_version: Some(1),
            rows_deleted: None,
            dropped_partitions: None,
            legacy_total_deleted: Some(7),
            batches: Some(4),
        });

        assert_eq!(response.detail_schema_version, Some(1));
        assert_eq!(response.rows_deleted, None);
        assert_eq!(response.dropped_partitions, None);
        assert_eq!(response.legacy_total_deleted, Some(7));
        assert_eq!(response.batches, Some(4));
    }
}
