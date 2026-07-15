use kernel::pagination::CursorPage;
use rbac::domain::DataScopeFilter;
use sqlx::{query, query_as};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;
use types::user::{ProfileUpdate, User, UserId, UserProfileGroups};

use crate::application::{AuthorizationUser, ReplaceUserRecord, UserListFilter};

use super::write;

mod audited_write;
pub(super) mod bootstrap;
mod cleanup;
mod cursor_page;
mod export;
mod options;
mod read_support;
mod relations;
mod write_support;

use self::{
    cleanup::delete_user_relations,
    write_support::{required_password, revoke_user_sessions, should_revoke_sessions},
};

#[derive(Clone)]
pub struct UserQueries {
    database: Database,
}

impl UserQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, input: ReplaceUserRecord) -> StorageResult<User> {
        self.ensure_references(&input).await?;
        let user_id = self.database.next_id();
        let password_hash = required_password(input.password_hash.clone())?;
        self.insert_user(&user_id, input, password_hash).await?;
        self.find_by_id(UserId(user_id)).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace(&self, id: UserId, input: ReplaceUserRecord) -> StorageResult<User> {
        self.ensure_references(&input).await?;
        self.update_user(&id, input).await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: UserId) -> StorageResult<()> {
        let user_id = id.0;
        let mut tx = self.database.pool().begin().await?;
        let result = query("UPDATE sys_user SET del_flag = '2', update_time = $2 WHERE user_id = $1 AND del_flag = '0'")
            .bind(&user_id)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *tx)
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        delete_user_relations(&mut tx, std::slice::from_ref(&user_id)).await?;
        revoke_user_sessions(&mut tx, &[user_id]).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn delete_many(&self, ids: Vec<UserId>) -> StorageResult<()> {
        let ids: Vec<String> = ids.into_iter().map(|id| id.0).collect();
        let mut tx = self.database.pool().begin().await?;
        let result = query("UPDATE sys_user SET del_flag = '2', update_time = $2 WHERE user_id = ANY($1) AND del_flag = '0'")
            .bind(&ids)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *tx)
            .await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        delete_user_relations(&mut tx, &ids).await?;
        revoke_user_sessions(&mut tx, &ids).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn find_by_id(&self, id: UserId) -> StorageResult<Option<User>> {
        self.find_record("user_id = $1", id.0.as_str()).await
    }

    pub async fn find_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        self.find_record("LOWER(email) = LOWER($1)", email).await
    }

    pub async fn find_by_phone(&self, phone: &str) -> StorageResult<Option<User>> {
        self.find_record("phonenumber = $1", phone).await
    }

    pub async fn find_auth_by_username(&self, username: &str) -> StorageResult<Option<(User, String)>> {
        self.find_auth_record("user_name = $1", username).await
    }

    pub async fn find_auth_by_email(&self, email: &str) -> StorageResult<Option<(User, String)>> {
        self.find_auth_record("LOWER(email) = LOWER($1)", email).await
    }

    pub async fn find_auth_by_id(&self, id: UserId) -> StorageResult<Option<(User, String)>> {
        self.find_auth_record("user_id = $1", &id.0).await
    }

    pub async fn find_authorization_by_id(&self, id: UserId) -> StorageResult<Option<AuthorizationUser>> {
        query_as::<_, super::record::AuthorizationUserRecord>(super::sql::authorization_user_query())
            .bind(id.0)
            .fetch_optional(self.database.pool())
            .await
            .map(|record| record.map(super::mapping::authorization_user))
            .map_err(StorageError::from)
    }

    pub async fn record_login(&self, id: UserId, ipaddr: String) -> StorageResult<()> {
        let now = OffsetDateTime::now_utc();
        let result = query("UPDATE sys_user SET login_ip = $2, login_date = $3, update_time = $3 WHERE user_id = $1 AND del_flag = '0'")
            .bind(id.0)
            .bind(ipaddr)
            .bind(now)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())
    }

    pub async fn list(&self, filter: UserListFilter) -> crate::application::AppResult<CursorPage<User>> {
        self.list_page(filter, None).await
    }

    pub async fn list_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> crate::application::AppResult<CursorPage<User>> {
        self.list_page(filter, Some(scope)).await
    }

    pub async fn update_password(&self, id: UserId, password_hash: String) -> StorageResult<()> {
        let user_id = id.0;
        let mut tx = self.database.pool().begin().await?;
        let result = query("UPDATE sys_user SET password=$2,pwd_update_date=CURRENT_TIMESTAMP,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&user_id)
            .bind(password_hash)
            .execute(&mut *tx)
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        revoke_user_sessions(&mut tx, &[user_id]).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn update_profile(&self, id: UserId, profile: ProfileUpdate) -> StorageResult<User> {
        let result = query("UPDATE sys_user SET nick_name=$2,email=$3,phonenumber=$4,sex=$5,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&id.0)
            .bind(profile.nick_name)
            .bind(profile.email)
            .bind(profile.phonenumber)
            .bind(profile.sex)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_avatar(&self, id: UserId, avatar: String) -> StorageResult<User> {
        let result = query("UPDATE sys_user SET avatar=$2,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&id.0)
            .bind(avatar)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_status(&self, id: UserId, status: String) -> StorageResult<User> {
        let revoke_sessions = should_revoke_sessions(false, &status);
        let mut tx = self.database.pool().begin().await?;
        let result = query("UPDATE sys_user SET status=$2,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&id.0)
            .bind(status)
            .execute(&mut *tx)
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        if revoke_sessions {
            revoke_user_sessions(&mut tx, std::slice::from_ref(&id.0)).await?;
        }
        tx.commit().await.map_err(StorageError::from)?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> StorageResult<User> {
        write::ensure_ids_exist(self.database.pool(), write::ReferenceTable::role(), &role_ids).await?;
        let mut tx = self.database.pool().begin().await?;
        write::replace_roles(&mut tx, &id.0, role_ids).await?;
        tx.commit().await.map_err(StorageError::from)?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn profile_groups(&self, id: UserId) -> StorageResult<UserProfileGroups> {
        let (role_group, post_group, dept_name) = tokio::try_join!(self.role_group(&id.0), self.post_group(&id.0), self.dept_name(&id.0),)?;
        Ok(UserProfileGroups {
            role_group,
            post_group,
            dept_name,
        })
    }
}

fn ensure_batch_rows(rows: u64, expected: usize) -> StorageResult<()> {
    if rows != expected as u64 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}
