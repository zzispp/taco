use audit_contract::AuditOutboxRecord;
use sqlx::{Postgres, Transaction};
use storage::{StorageError, StorageResult, outbox::append_audit_record};

pub(super) async fn commit_audited_write(mut transaction: Transaction<'_, Postgres>, audit: &AuditOutboxRecord) -> StorageResult<()> {
    match append_audit_record(&mut transaction, audit).await {
        Ok(()) => transaction.commit().await.map_err(StorageError::from),
        Err(error) => {
            transaction.rollback().await.map_err(StorageError::from)?;
            Err(error)
        }
    }
}
