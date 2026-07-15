use async_trait::async_trait;
use kernel::pagination::CursorPage;
use sqlx::{FromRow, query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::application::{AppError, AppResult, OnlineSession, OnlineSessionCleanup, OnlineSessionPageRequest, OnlineSessionStore};
use crate::domain::UserId;

const ACTIVE_USER_STATUS: &str = "0";
const NOT_DELETED_FLAG: &str = "0";
const NANOS_PER_MILLISECOND: i128 = 1_000_000;

mod pagination;

#[derive(Clone)]
pub struct StorageOnlineSessionStore {
    database: Database,
}

#[derive(FromRow)]
struct OnlineSessionRecord {
    token_id: String,
    user_id: String,
    dept_id: Option<String>,
    dept_name: Option<String>,
    user_name: String,
    ipaddr: String,
    login_location: String,
    browser: String,
    os: String,
    login_time: OffsetDateTime,
    expires_at: OffsetDateTime,
}

impl StorageOnlineSessionStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl OnlineSessionStore for StorageOnlineSessionStore {
    async fn create(&self, session: &OnlineSession) -> AppResult<()> {
        let result = query(
            "INSERT INTO sys_user_session (token_id,user_id,dept_name,user_name,ipaddr,login_location,browser,os,login_time,expires_at) \
             SELECT $1,$2,$3,$4,$5,$6,$7,$8,$9,$10 FROM sys_user \
             WHERE user_id=$2 AND status=$11 AND del_flag=$12",
        )
        .bind(&session.token_id)
        .bind(&session.user_id.0)
        .bind(&session.dept_name)
        .bind(&session.user_name)
        .bind(&session.ipaddr)
        .bind(&session.login_location)
        .bind(&session.browser)
        .bind(&session.os)
        .bind(from_epoch_millis(session.login_time)?)
        .bind(from_epoch_millis(session.expires_at)?)
        .bind(ACTIVE_USER_STATUS)
        .bind(NOT_DELETED_FLAG)
        .execute(self.database.pool())
        .await
        .map_err(storage_error)?;
        if result.rows_affected() != 1 {
            return Err(AppError::Unauthorized);
        }
        Ok(())
    }

    async fn renew_active(&self, token_id: &str, user_id: &UserId, expires_at: i64) -> AppResult<Option<OnlineSession>> {
        let record = query_as::<_, OnlineSessionRecord>(
            "UPDATE sys_user_session AS session SET expires_at=$3 FROM sys_user AS users \
             WHERE session.token_id=$1 AND session.user_id=$2 AND session.expires_at>CURRENT_TIMESTAMP \
             AND users.user_id=session.user_id AND users.status=$4 AND users.del_flag=$5 \
             RETURNING session.token_id,session.user_id,users.dept_id,session.dept_name,session.user_name,session.ipaddr,\
             session.login_location,session.browser,session.os,session.login_time,session.expires_at",
        )
        .bind(token_id)
        .bind(&user_id.0)
        .bind(from_epoch_millis(expires_at)?)
        .bind(ACTIVE_USER_STATUS)
        .bind(NOT_DELETED_FLAG)
        .fetch_optional(self.database.pool())
        .await
        .map_err(storage_error)?;
        record.map(OnlineSession::try_from).transpose()
    }

    async fn find_active(&self, token_id: &str, user_id: &UserId) -> AppResult<Option<OnlineSession>> {
        let record = query_as::<_, OnlineSessionRecord>(
            "SELECT session.token_id,session.user_id,users.dept_id,session.dept_name,session.user_name,session.ipaddr,\
             session.login_location,session.browser,session.os,session.login_time,session.expires_at \
             FROM sys_user_session AS session JOIN sys_user AS users ON users.user_id=session.user_id \
             WHERE session.token_id=$1 AND session.user_id=$2 AND session.expires_at>CURRENT_TIMESTAMP \
             AND users.status=$3 AND users.del_flag=$4",
        )
        .bind(token_id)
        .bind(&user_id.0)
        .bind(ACTIVE_USER_STATUS)
        .bind(NOT_DELETED_FLAG)
        .fetch_optional(self.database.pool())
        .await
        .map_err(storage_error)?;
        record.map(OnlineSession::try_from).transpose()
    }

    async fn find_active_by_token(&self, token_id: &str) -> AppResult<Option<OnlineSession>> {
        let record = query_as::<_, OnlineSessionRecord>(
            "SELECT session.token_id,session.user_id,users.dept_id,session.dept_name,session.user_name,session.ipaddr,\
             session.login_location,session.browser,session.os,session.login_time,session.expires_at \
             FROM sys_user_session AS session JOIN sys_user AS users ON users.user_id=session.user_id \
             WHERE session.token_id=$1 AND session.expires_at>CURRENT_TIMESTAMP \
             AND users.status=$2 AND users.del_flag=$3",
        )
        .bind(token_id)
        .bind(ACTIVE_USER_STATUS)
        .bind(NOT_DELETED_FLAG)
        .fetch_optional(self.database.pool())
        .await
        .map_err(storage_error)?;
        record.map(OnlineSession::try_from).transpose()
    }

    async fn delete(&self, token_id: &str) -> AppResult<()> {
        query("DELETE FROM sys_user_session WHERE token_id=$1")
            .bind(token_id)
            .execute(self.database.pool())
            .await
            .map_err(storage_error)?;
        Ok(())
    }

    async fn page_active(&self, request: OnlineSessionPageRequest) -> AppResult<CursorPage<OnlineSession>> {
        pagination::page_active(self, request).await
    }
}

#[async_trait]
impl OnlineSessionCleanup for StorageOnlineSessionStore {
    async fn delete_expired(&self, batch_size: usize) -> AppResult<u64> {
        let limit =
            i64::try_from(batch_size).map_err(|error| AppError::Infrastructure(format!("online session cleanup batch size conversion failed: {error}")))?;
        let result = query(
            "WITH expired AS (SELECT token_id FROM sys_user_session WHERE expires_at<=CURRENT_TIMESTAMP \
             ORDER BY expires_at ASC,token_id ASC LIMIT $1 FOR UPDATE SKIP LOCKED) \
             DELETE FROM sys_user_session AS session USING expired WHERE session.token_id=expired.token_id",
        )
        .bind(limit)
        .execute(self.database.pool())
        .await
        .map_err(storage_error)?;
        Ok(result.rows_affected())
    }
}

impl TryFrom<OnlineSessionRecord> for OnlineSession {
    type Error = AppError;

    fn try_from(value: OnlineSessionRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            token_id: value.token_id,
            user_id: UserId(value.user_id),
            dept_id: value.dept_id,
            dept_name: value.dept_name,
            user_name: value.user_name,
            ipaddr: value.ipaddr,
            login_location: value.login_location,
            browser: value.browser,
            os: value.os,
            login_time: to_epoch_millis(value.login_time)?,
            expires_at: to_epoch_millis(value.expires_at)?,
        })
    }
}

fn from_epoch_millis(value: i64) -> AppResult<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value) * NANOS_PER_MILLISECOND)
        .map_err(|error| AppError::Infrastructure(format!("online session timestamp error: {error}")))
}

fn to_epoch_millis(value: OffsetDateTime) -> AppResult<i64> {
    i64::try_from(value.unix_timestamp_nanos() / NANOS_PER_MILLISECOND)
        .map_err(|error| AppError::Infrastructure(format!("online session timestamp overflow: {error}")))
}

fn storage_error(error: sqlx::Error) -> AppError {
    AppError::Infrastructure(format!("online session storage error: {error}"))
}

fn storage_mapping_error(error: storage::StorageError) -> AppError {
    AppError::Infrastructure(error.to_string())
}
