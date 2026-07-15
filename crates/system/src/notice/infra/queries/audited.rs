use audit_contract::AuditOutboxRecord;
use constants::system::STATUS_NORMAL;
use sqlx::{Postgres, Transaction, query, query_scalar};
use storage::{StorageError, StorageResult, outbox::append_audit_record};
use time::OffsetDateTime;

use crate::notice::domain::{Notice, NoticeInput, ReplaceNoticeCommand};

use super::{MARK_ALL_READ_SQL, MARK_READ_SQL, NoticeQueries, UNREAD_NOTICE_IDS_SQL, ensure_rows, next_read_ids};

impl NoticeQueries {
    pub(in crate::notice::infra) async fn create_notice_with_audit(
        &self,
        input: NoticeInput,
        operator: &str,
        audit: &AuditOutboxRecord,
    ) -> StorageResult<Notice> {
        let id = self.database.next_id();
        let mut transaction = self.database.pool().begin().await?;
        query(
            "INSERT INTO sys_notice (notice_id,notice_title,notice_type,notice_content,status,create_by,create_time,remark) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(&id)
        .bind(input.notice_title)
        .bind(input.notice_type)
        .bind(input.notice_content)
        .bind(input.status)
        .bind(operator)
        .bind(OffsetDateTime::now_utc())
        .bind(input.remark)
        .execute(&mut *transaction)
        .await?;
        commit_audited_write(transaction, audit).await?;
        self.find_notice(&id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::notice::infra) async fn replace_notice_with_audit(&self, command: &ReplaceNoticeCommand, audit: &AuditOutboxRecord) -> StorageResult<Notice> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query(
            "UPDATE sys_notice SET notice_title=$2,notice_type=$3,notice_content=$4,status=$5,update_by=$6,update_time=CURRENT_TIMESTAMP,remark=$7 WHERE notice_id=$1",
        )
        .bind(&command.id)
        .bind(&command.input.notice_title)
        .bind(&command.input.notice_type)
        .bind(&command.input.notice_content)
        .bind(&command.input.status)
        .bind(&command.operator)
        .bind(&command.input.remark)
        .execute(&mut *transaction)
        .await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find_notice(&command.id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::notice::infra) async fn delete_notice_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_notice WHERE notice_id=$1").bind(id).execute(&mut *transaction).await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::notice::infra) async fn delete_notices_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_notice WHERE notice_id=ANY($1)")
            .bind(ids)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::notice::infra) async fn mark_read_with_audit(&self, notice_id: &str, user_id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let status = query_scalar::<_, String>("SELECT status FROM sys_notice WHERE notice_id=$1 FOR UPDATE")
            .bind(notice_id)
            .fetch_optional(&mut *transaction)
            .await?;
        if status.as_deref() != Some(STATUS_NORMAL) {
            return Err(StorageError::NotFound);
        }
        query(MARK_READ_SQL)
            .bind(self.database.next_id())
            .bind(notice_id)
            .bind(user_id)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::notice::infra) async fn mark_all_read_with_audit(&self, user_id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let notice_ids = query_scalar::<_, String>(UNREAD_NOTICE_IDS_SQL)
            .bind(user_id)
            .bind(STATUS_NORMAL)
            .fetch_all(&mut *transaction)
            .await?;
        if !notice_ids.is_empty() {
            let read_ids = next_read_ids(&self.database, notice_ids.len());
            query(MARK_ALL_READ_SQL)
                .bind(read_ids)
                .bind(notice_ids)
                .bind(user_id)
                .bind(OffsetDateTime::now_utc())
                .execute(&mut *transaction)
                .await?;
        }
        commit_audited_write(transaction, audit).await
    }
}

async fn commit_audited_write(mut transaction: Transaction<'_, Postgres>, audit: &AuditOutboxRecord) -> StorageResult<()> {
    match append_audit_record(&mut transaction, audit).await {
        Ok(()) => transaction.commit().await.map_err(StorageError::from),
        Err(error) => {
            transaction.rollback().await.map_err(StorageError::from)?;
            Err(error)
        }
    }
}
