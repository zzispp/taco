pub type AuditOutboxResult<T> = Result<T, AuditOutboxError>;

#[derive(Debug, thiserror::Error)]
pub enum AuditOutboxError {
    #[error("invalid audit outbox payload: {0}")]
    InvalidPayload(String),
    #[error("audit outbox infrastructure failure: {0}")]
    Infrastructure(String),
}
