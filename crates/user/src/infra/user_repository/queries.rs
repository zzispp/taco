use constants::pagination::PAGE_INDEX_OFFSET;
use kernel::pagination::{Page, PageSliceRequest};
use sqlx::query;
use storage::{Database, StorageError, StorageResult, database::to_u64};
use time::OffsetDateTime;
use types::{
    rbac::DataScopeFilter,
    user::{ProfileUpdate, User, UserId, UserProfileGroups},
};

use crate::application::{ReplaceUserRecord, UserListFilter};

use super::write;

mod options;
mod read_support;
mod write_support;

use self::write_support::required_password;

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
        let result = query("UPDATE sys_user SET del_flag = '2', update_time = $2 WHERE user_id = $1 AND del_flag = '0'")
            .bind(id.0)
            .bind(OffsetDateTime::now_utc())
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())
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
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn find_by_id(&self, id: UserId) -> StorageResult<Option<User>> {
        self.find_record("user_id = $1", id.0.as_str()).await
    }

    pub async fn find_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        self.find_record("email = $1", email).await
    }

    pub async fn find_by_phone(&self, phone: &str) -> StorageResult<Option<User>> {
        self.find_record("phonenumber = $1", phone).await
    }

    pub async fn find_auth_by_username(&self, username: &str) -> StorageResult<Option<(User, String)>> {
        self.find_auth_record("user_name = $1", username).await
    }

    pub async fn find_auth_by_email(&self, email: &str) -> StorageResult<Option<(User, String)>> {
        self.find_auth_record("email = $1", email).await
    }

    pub async fn find_auth_by_id(&self, id: UserId) -> StorageResult<Option<(User, String)>> {
        self.find_auth_record("user_id = $1", &id.0).await
    }

    pub async fn record_login(&self, id: UserId) -> StorageResult<()> {
        let now = OffsetDateTime::now_utc();
        let result = query("UPDATE sys_user SET login_date = $2, update_time = $2 WHERE user_id = $1 AND del_flag = '0'")
            .bind(id.0)
            .bind(now)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())
    }

    pub async fn list(&self, filter: UserListFilter) -> StorageResult<Page<User>> {
        let page = filter.page;
        let request = PageSliceRequest {
            offset: (page.page - PAGE_INDEX_OFFSET) * page.page_size,
            limit: page.page_size,
            page: page.page,
            page_size: page.page_size,
        };
        self.list_slice(filter, request).await
    }

    pub async fn list_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> StorageResult<Page<User>> {
        let page = filter.page;
        let offset = (page.page - PAGE_INDEX_OFFSET) * page.page_size;
        let ids = self
            .scoped_user_ids(&filter, &scope, read_support::ScopedUserSlice { limit: page.page_size, offset })
            .await?;
        let total = self.scoped_user_total(&filter, &scope).await?;
        Ok(Page {
            items: self.users_by_ids(ids).await?,
            total: to_u64(total)?,
            page: page.page,
            page_size: page.page_size,
        })
    }

    pub async fn update_password(&self, id: UserId, password_hash: String) -> StorageResult<()> {
        let result = query("UPDATE sys_user SET password=$2,pwd_update_date=CURRENT_TIMESTAMP,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(id.0)
            .bind(password_hash)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())
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
        let result = query("UPDATE sys_user SET status=$2,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&id.0)
            .bind(status)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
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
        Ok(UserProfileGroups {
            role_group: self.role_group(&id.0).await?,
            post_group: self.post_group(&id.0).await?,
            dept_name: self.dept_name(&id.0).await?,
        })
    }
}

fn ensure_batch_rows(rows: u64, expected: usize) -> StorageResult<()> {
    if rows != expected as u64 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}
