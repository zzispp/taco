use storage::{StorageError, StorageResult};

pub(super) fn ensure_rows_affected(rows_affected: u64) -> StorageResult<()> {
    if rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub(super) fn ensure_batch_rows(rows: u64, expected: usize) -> StorageResult<()> {
    if rows != expected as u64 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}
