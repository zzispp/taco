use kernel::error::LocalizedError;

use super::SystemLogRetentionReport;

pub type ObservabilityResult<T> = Result<T, ObservabilityError>;

#[derive(Debug, thiserror::Error)]
pub enum ObservabilityError {
    #[error("system log not found")]
    NotFound,
    #[error("invalid system log input: {0}")]
    InvalidInput(LocalizedError),
    #[error("invalid system log cursor")]
    InvalidCursor,
    #[error("{code}: {details}")]
    Conflict { code: &'static str, details: LocalizedError },
    #[error(
        "system log cleanup partially completed after {rows_deleted} row deletions and {dropped_partitions} partition drops in {batches} batches: {failure}"
    )]
    PartialCleanup {
        rows_deleted: u64,
        dropped_partitions: u64,
        batches: u64,
        failure: String,
    },
    #[error("system log infrastructure failure: {0}")]
    Infrastructure(String),
}

impl ObservabilityError {
    pub fn conflict(code: &'static str, details: LocalizedError) -> Self {
        Self::Conflict { code, details }
    }

    pub fn partial_cleanup(report: SystemLogRetentionReport, failure: impl Into<String>) -> Self {
        Self::PartialCleanup {
            rows_deleted: report.rows_deleted,
            dropped_partitions: report.dropped_partitions,
            batches: report.batches,
            failure: failure.into(),
        }
    }
}

pub fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

pub fn localized_param(key: &'static str, name: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(name, value)
}

#[cfg(test)]
mod tests {
    use super::ObservabilityError;
    use crate::application::SystemLogRetentionReport;

    #[test]
    fn partial_cleanup_retains_each_completed_work_unit_separately() {
        let error = ObservabilityError::partial_cleanup(
            SystemLogRetentionReport {
                rows_deleted: 7,
                dropped_partitions: 2,
                batches: 3,
            },
            "planned failure",
        );

        assert!(matches!(
            error,
            ObservabilityError::PartialCleanup {
                rows_deleted: 7,
                dropped_partitions: 2,
                batches: 3,
                ..
            }
        ));
    }
}
