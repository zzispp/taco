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
    #[error("system log cleanup partially completed after {deleted} deletions in {batches} batches: {failure}")]
    PartialCleanup { deleted: u64, batches: u64, failure: String },
    #[error("system log infrastructure failure: {0}")]
    Infrastructure(String),
}

impl ObservabilityError {
    pub fn conflict(code: &'static str, details: LocalizedError) -> Self {
        Self::Conflict { code, details }
    }

    pub fn partial_cleanup(report: SystemLogRetentionReport, failure: impl Into<String>) -> Self {
        Self::PartialCleanup {
            deleted: report.deleted,
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
