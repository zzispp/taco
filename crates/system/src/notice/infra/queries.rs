use constants::system::STATUS_NORMAL;
use kernel::pagination::{Page, PageRequest};
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;

use super::records::{NoticeReaderRecord, NoticeRecord, NoticeSummaryRecord, NoticeTopRecord};
use crate::notice::domain::{Notice, NoticeInput, NoticeListFilter, NoticeReader, NoticeReaderFilter, NoticeSummary, NoticeTopItem, NoticeTopResponse};

const NOTICE_COLUMNS: &str =
    "notice_id,notice_title,notice_type,notice_content,status,create_by,create_time::text AS create_time,update_by,update_time::text AS update_time,remark";
const NOTICE_SUMMARY_COLUMNS: &str = "notice_id,notice_title,notice_type,status,create_by,create_time::text AS create_time";
const NOTICE_PREDICATE: &str = "($1::text IS NULL OR notice_title ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR create_by ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR notice_type=$3)";
const READER_PREDICATE: &str =
    "r.notice_id=$1 AND u.del_flag='0' AND ($2::text IS NULL OR u.user_name ILIKE '%' || $2 || '%' OR u.nick_name ILIKE '%' || $2 || '%')";
const MARK_READ_SQL: &str = "INSERT INTO sys_notice_read (read_id,notice_id,user_id,read_time) VALUES ($1,$2,$3,$4) ON CONFLICT ON CONSTRAINT uk_sys_notice_read_user_notice DO NOTHING";
const MARK_ALL_READ_SQL: &str = "INSERT INTO sys_notice_read (read_id,notice_id,user_id,read_time) SELECT batch.read_id,batch.notice_id,$3,$4 FROM UNNEST($1::text[],$2::text[]) AS batch(read_id,notice_id) ON CONFLICT ON CONSTRAINT uk_sys_notice_read_user_notice DO NOTHING";
const UNREAD_NOTICE_IDS_SQL: &str = "SELECT notice_id FROM sys_notice n WHERE status=$2 AND NOT EXISTS (SELECT 1 FROM sys_notice_read r WHERE r.notice_id=n.notice_id AND r.user_id=$1) ORDER BY notice_id FOR UPDATE OF n";

#[derive(Clone)]
pub struct NoticeQueries {
    database: Database,
}

impl NoticeQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn page_notices(&self, filter: NoticeListFilter) -> StorageResult<Page<NoticeSummary>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(format!("SELECT COUNT(*) FROM sys_notice WHERE {NOTICE_PREDICATE}")))
            .bind(&filter.notice_title)
            .bind(&filter.create_by)
            .bind(&filter.notice_type)
            .fetch_one(self.database.pool())
            .await?;
        let sql =
            format!("SELECT {NOTICE_SUMMARY_COLUMNS} FROM sys_notice WHERE {NOTICE_PREDICATE} ORDER BY create_time DESC, notice_id DESC LIMIT $4 OFFSET $5");
        let rows = query_as::<_, NoticeSummaryRecord>(AssertSqlSafe(sql))
            .bind(&filter.notice_title)
            .bind(&filter.create_by)
            .bind(&filter.notice_type)
            .bind(limit(filter.page)?)
            .bind(offset(filter.page)?)
            .fetch_all(self.database.pool())
            .await?;
        page(rows.into_iter().map(NoticeSummary::from).collect(), total, filter.page)
    }

    pub async fn find_notice(&self, id: &str) -> StorageResult<Option<Notice>> {
        query_as::<_, NoticeRecord>(AssertSqlSafe(format!("SELECT {NOTICE_COLUMNS} FROM sys_notice WHERE notice_id=$1")))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map(|record| record.map(Notice::from))
            .map_err(StorageError::from)
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
        let rows = query_as::<_, NoticeTopRecord>("SELECT n.notice_id,n.notice_title,n.notice_type,n.create_by,n.create_time::text AS create_time,EXISTS (SELECT 1 FROM sys_notice_read r WHERE r.notice_id=n.notice_id AND r.user_id=$1) AS is_read FROM sys_notice n WHERE n.status=$2 ORDER BY n.create_time DESC,n.notice_id DESC LIMIT $3")
            .bind(user_id)
            .bind(STATUS_NORMAL)
            .bind(storage::database::to_i64(limit)?)
            .fetch_all(self.database.pool())
            .await?;
        Ok(NoticeTopResponse {
            items: rows.into_iter().map(NoticeTopItem::from).collect(),
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

    pub async fn page_readers(&self, notice_id: &str, filter: NoticeReaderFilter) -> StorageResult<Page<NoticeReader>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM sys_notice_read r JOIN sys_user u ON u.user_id=r.user_id WHERE {READER_PREDICATE}"
        )))
        .bind(notice_id)
        .bind(&filter.search_value)
        .fetch_one(self.database.pool())
        .await?;
        let rows = query_as::<_, NoticeReaderRecord>(AssertSqlSafe(format!("SELECT r.user_id,u.user_name,u.nick_name,d.dept_name,u.phonenumber,r.read_time::text AS read_time FROM sys_notice_read r JOIN sys_user u ON u.user_id=r.user_id LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE {READER_PREDICATE} ORDER BY r.read_time DESC,r.user_id DESC LIMIT $3 OFFSET $4")))
            .bind(notice_id)
            .bind(&filter.search_value)
            .bind(limit(filter.page)?)
            .bind(offset(filter.page)?)
            .fetch_all(self.database.pool())
            .await?;
        page(rows.into_iter().map(NoticeReader::from).collect(), total, filter.page)
    }
}

fn limit(page: PageRequest) -> StorageResult<i64> {
    storage::database::to_i64(page.page_size)
}

fn offset(page: PageRequest) -> StorageResult<i64> {
    let offset = page
        .page
        .checked_sub(1)
        .and_then(|value| value.checked_mul(page.page_size))
        .ok_or_else(|| StorageError::Database("pagination overflow".into()))?;
    storage::database::to_i64(offset)
}

fn page<T>(items: Vec<T>, total: i64, request: PageRequest) -> StorageResult<Page<T>> {
    Ok(Page {
        items,
        total: storage::database::to_u64(total)?,
        page: request.page,
        page_size: request.page_size,
    })
}

fn ensure_rows(rows: u64) -> StorageResult<()> {
    if rows == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

fn next_read_ids(database: &Database, count: usize) -> Vec<String> {
    (0..count).map(|_| database.next_id()).collect()
}

impl From<NoticeRecord> for Notice {
    fn from(value: NoticeRecord) -> Self {
        Self {
            notice_id: value.notice_id,
            notice_title: value.notice_title,
            notice_type: value.notice_type,
            notice_content: value.notice_content,
            status: value.status,
            create_by: value.create_by,
            create_time: value.create_time,
            update_by: value.update_by,
            update_time: value.update_time,
            remark: value.remark,
        }
    }
}

impl From<NoticeSummaryRecord> for NoticeSummary {
    fn from(value: NoticeSummaryRecord) -> Self {
        Self {
            notice_id: value.notice_id,
            notice_title: value.notice_title,
            notice_type: value.notice_type,
            status: value.status,
            create_by: value.create_by,
            create_time: value.create_time,
        }
    }
}

impl From<NoticeTopRecord> for NoticeTopItem {
    fn from(value: NoticeTopRecord) -> Self {
        Self {
            notice_id: value.notice_id,
            notice_title: value.notice_title,
            notice_type: value.notice_type,
            create_by: value.create_by,
            create_time: value.create_time,
            is_read: value.is_read,
        }
    }
}

impl From<NoticeReaderRecord> for NoticeReader {
    fn from(value: NoticeReaderRecord) -> Self {
        Self {
            user_id: value.user_id,
            user_name: value.user_name,
            nick_name: value.nick_name,
            dept_name: value.dept_name,
            phonenumber: value.phonenumber,
            read_time: value.read_time,
        }
    }
}

#[cfg(test)]
mod tests;
