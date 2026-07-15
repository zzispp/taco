use constants::system::STATUS_NORMAL;
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;

use crate::application::SystemResult;
use crate::notice::domain::{Notice, NoticeInput, NoticeListFilter, NoticeReader, NoticeReaderFilter, NoticeSummary, NoticeTopResponse};

use super::mapping::{notice, top};
use super::records::{NoticeRecord, NoticeTopRecord};

pub(super) const NOTICE_COLUMNS: &str = "notice_id,notice_title,notice_type,notice_content,status,create_by,create_time,update_by,update_time,remark";
pub(super) const NOTICE_SUMMARY_COLUMNS: &str = "notice_id,notice_title,notice_type,status,create_by,create_time";
const MARK_READ_SQL: &str = "INSERT INTO sys_notice_read (read_id,notice_id,user_id,read_time) VALUES ($1,$2,$3,$4) ON CONFLICT ON CONSTRAINT uk_sys_notice_read_user_notice DO NOTHING";
const MARK_ALL_READ_SQL: &str = "INSERT INTO sys_notice_read (read_id,notice_id,user_id,read_time) SELECT batch.read_id,batch.notice_id,$3,$4 FROM UNNEST($1::text[],$2::text[]) AS batch(read_id,notice_id) ON CONFLICT ON CONSTRAINT uk_sys_notice_read_user_notice DO NOTHING";
const UNREAD_NOTICE_IDS_SQL: &str = "SELECT notice_id FROM sys_notice n WHERE status=$2 AND NOT EXISTS (SELECT 1 FROM sys_notice_read r WHERE r.notice_id=n.notice_id AND r.user_id=$1) ORDER BY notice_id FOR UPDATE OF n";

#[derive(Clone)]
pub struct NoticeQueries {
    pub(super) database: Database,
}

impl NoticeQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn page_notices(&self, filter: NoticeListFilter) -> SystemResult<kernel::pagination::CursorPage<NoticeSummary>> {
        pages::page_notices(&self.database, filter).await
    }

    pub async fn find_notice(&self, id: &str) -> StorageResult<Option<Notice>> {
        query_as::<_, NoticeRecord>(AssertSqlSafe(format!("SELECT {NOTICE_COLUMNS} FROM sys_notice WHERE notice_id=$1")))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)?
            .map(notice)
            .transpose()
    }

    pub async fn create_notice(&self, input: NoticeInput, operator: &str) -> StorageResult<Notice> {
        let id = self.database.next_id();
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
        .execute(self.database.pool())
        .await?;
        self.find_notice(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace_notice(&self, id: &str, input: NoticeInput, operator: &str) -> StorageResult<Notice> {
        let result = query("UPDATE sys_notice SET notice_title=$2,notice_type=$3,notice_content=$4,status=$5,update_by=$6,update_time=CURRENT_TIMESTAMP,remark=$7 WHERE notice_id=$1")
            .bind(id)
            .bind(input.notice_title)
            .bind(input.notice_type)
            .bind(input.notice_content)
            .bind(input.status)
            .bind(operator)
            .bind(input.remark)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())?;
        self.find_notice(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete_notice(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM sys_notice WHERE notice_id=$1")
            .bind(id)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())
    }

    pub async fn delete_notices(&self, ids: &[String]) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_notice WHERE notice_id=ANY($1)")
            .bind(ids)
            .execute(&mut *transaction)
            .await?;
        if result.rows_affected() != ids.len() as u64 {
            return Err(StorageError::NotFound);
        }
        transaction.commit().await.map_err(StorageError::from)
    }

    pub async fn top_notices(&self, user_id: &str, limit: u64) -> StorageResult<NoticeTopResponse> {
        let unread_count = query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_notice n WHERE n.status=$2 AND NOT EXISTS (SELECT 1 FROM sys_notice_read r WHERE r.notice_id=n.notice_id AND r.user_id=$1)")
            .bind(user_id)
            .bind(STATUS_NORMAL)
            .fetch_one(self.database.pool())
            .await?;
        let rows = query_as::<_, NoticeTopRecord>("SELECT n.notice_id,n.notice_title,n.notice_type,n.create_by,n.create_time,EXISTS (SELECT 1 FROM sys_notice_read r WHERE r.notice_id=n.notice_id AND r.user_id=$1) AS is_read FROM sys_notice n WHERE n.status=$2 ORDER BY n.create_time DESC,n.notice_id DESC LIMIT $3")
            .bind(user_id)
            .bind(STATUS_NORMAL)
            .bind(storage::database::to_i64(limit)?)
            .fetch_all(self.database.pool())
            .await?;
        Ok(NoticeTopResponse {
            items: rows.into_iter().map(top).collect::<StorageResult<Vec<_>>>()?,
            unread_count: storage::database::to_u64(unread_count)?,
        })
    }

    pub async fn mark_read(&self, notice_id: &str, user_id: &str) -> StorageResult<()> {
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
        transaction.commit().await.map_err(StorageError::from)
    }

    pub async fn mark_all_read(&self, user_id: &str) -> StorageResult<()> {
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
        transaction.commit().await.map_err(StorageError::from)
    }

    pub async fn page_readers(&self, notice_id: &str, filter: NoticeReaderFilter) -> SystemResult<kernel::pagination::CursorPage<NoticeReader>> {
        pages::page_readers(&self.database, notice_id, filter).await
    }
}

pub(super) fn ensure_rows(rows: u64) -> StorageResult<()> {
    if rows == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub(super) fn next_read_ids(database: &Database, count: usize) -> Vec<String> {
    (0..count).map(|_| database.next_id()).collect()
}

mod audited;
mod pages;

#[cfg(test)]
mod tests;
