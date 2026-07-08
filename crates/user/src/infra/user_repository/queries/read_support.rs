use kernel::pagination::{Page, PageSliceRequest};
use sqlx::{AssertSqlSafe, query_as, query_scalar};
use storage::{
    StorageError, StorageResult,
    database::{to_i64, to_u64},
};
use types::{
    rbac::DataScopeFilter,
    user::{User, UserId},
};

use crate::application::UserListFilter;

use super::UserQueries;
use crate::infra::user_repository::{
    filter_sql,
    mapping::{UserRelations, role_summary, user},
    record::{RoleSummaryRecord, UserRecord},
    sql,
};

pub(super) struct ScopedUserSlice {
    pub(super) limit: u64,
    pub(super) offset: u64,
}

impl UserQueries {
    pub async fn list_slice(&self, filter: UserListFilter, request: PageSliceRequest) -> StorageResult<Page<User>> {
        let total = self.filtered_total(&filter).await?;
        let records = self.filtered_records_slice(&filter, request.limit, request.offset).await?;
        Ok(Page {
            items: self.users(records).await?,
            total: to_u64(total)?,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn users(&self, records: Vec<UserRecord>) -> StorageResult<Vec<User>> {
        let mut users = Vec::with_capacity(records.len());
        for record in records {
            let relations = self.relations(&record.user_id).await?;
            users.push(user(record, relations));
        }
        Ok(users)
    }

    pub(super) async fn users_by_ids(&self, ids: Vec<String>) -> StorageResult<Vec<User>> {
        let mut users = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(user) = self.find_by_id(UserId(id)).await? {
                users.push(user);
            }
        }
        Ok(users)
    }

    async fn filtered_total(&self, filter: &UserListFilter) -> StorageResult<i64> {
        query_scalar(filter_sql::filtered_users_total())
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(&filter.status)
            .bind(&filter.dept_id)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(&filter.nick_name)
            .bind(&filter.dept_name)
            .bind(&filter.email)
            .bind(&filter.sex)
            .bind(&filter.post_ids)
            .bind(&filter.role_ids)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn filtered_records_slice(&self, filter: &UserListFilter, limit: u64, offset: u64) -> StorageResult<Vec<UserRecord>> {
        let sql = filter_sql::filtered_users("ORDER BY u.create_time ASC LIMIT $13 OFFSET $14");
        query_as::<_, UserRecord>(AssertSqlSafe(sql))
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(&filter.status)
            .bind(&filter.dept_id)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(&filter.nick_name)
            .bind(&filter.dept_name)
            .bind(&filter.email)
            .bind(&filter.sex)
            .bind(&filter.post_ids)
            .bind(&filter.role_ids)
            .bind(to_i64(limit)?)
            .bind(to_i64(offset)?)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub(super) async fn scoped_user_ids(&self, filter: &UserListFilter, scope: &DataScopeFilter, slice: ScopedUserSlice) -> StorageResult<Vec<String>> {
        query_scalar(filter_sql::scoped_user_ids())
            .bind(&scope.data_scope)
            .bind(&scope.user_id)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(&filter.status)
            .bind(&filter.dept_id)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(&filter.nick_name)
            .bind(&filter.dept_name)
            .bind(&filter.email)
            .bind(&filter.sex)
            .bind(&filter.post_ids)
            .bind(&filter.role_ids)
            .bind(to_i64(slice.limit)?)
            .bind(to_i64(slice.offset)?)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub(super) async fn scoped_user_total(&self, filter: &UserListFilter, scope: &DataScopeFilter) -> StorageResult<i64> {
        query_scalar(filter_sql::scoped_user_total())
            .bind(&scope.data_scope)
            .bind(&scope.user_id)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(&filter.status)
            .bind(&filter.dept_id)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(&filter.nick_name)
            .bind(&filter.dept_name)
            .bind(&filter.email)
            .bind(&filter.sex)
            .bind(&filter.post_ids)
            .bind(&filter.role_ids)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn scoped_existing_user_ids(&self, ids: Vec<String>, scope: &DataScopeFilter) -> StorageResult<Vec<String>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        query_scalar(sql::scoped_existing_user_ids())
            .bind(&scope.data_scope)
            .bind(&scope.user_id)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .bind(&ids)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub(super) async fn find_record(&self, predicate: &'static str, value: &str) -> StorageResult<Option<User>> {
        let record = self.raw_record(predicate, value).await?;
        match record {
            Some(record) => self.user(record).await.map(Some),
            None => Ok(None),
        }
    }

    pub(super) async fn find_auth_record(&self, predicate: &'static str, value: &str) -> StorageResult<Option<(User, String)>> {
        let record = self.raw_record(predicate, value).await?;
        match record {
            Some(record) => self.auth_user(record).await.map(Some),
            None => Ok(None),
        }
    }

    async fn raw_record(&self, predicate: &'static str, value: &str) -> StorageResult<Option<UserRecord>> {
        query_as::<_, UserRecord>(AssertSqlSafe(format!(
            "SELECT {} FROM sys_user WHERE del_flag = '0' AND {predicate}",
            sql::USER_COLUMNS
        )))
        .bind(value)
        .fetch_optional(self.database.pool())
        .await
        .map_err(StorageError::from)
    }

    async fn auth_user(&self, record: UserRecord) -> StorageResult<(User, String)> {
        let password = record.password.clone();
        Ok((self.user(record).await?, password))
    }

    async fn user(&self, record: UserRecord) -> StorageResult<User> {
        let relations = self.relations(&record.user_id).await?;
        Ok(user(record, relations))
    }

    async fn relations(&self, user_id: &str) -> StorageResult<UserRelations> {
        let roles = self.roles(user_id).await?;
        Ok(UserRelations {
            role_ids: roles.iter().map(|role| role.role_id.clone()).collect(),
            roles,
            post_ids: self.post_ids(user_id).await?,
            permissions: self.permissions(user_id).await?,
        })
    }

    async fn roles(&self, user_id: &str) -> StorageResult<Vec<types::rbac::RoleSummary>> {
        query_as::<_, RoleSummaryRecord>(sql::role_query())
            .bind(user_id)
            .fetch_all(self.database.pool())
            .await
            .map(|records| records.into_iter().map(role_summary).collect())
            .map_err(StorageError::from)
    }

    pub(super) async fn role_group(&self, user_id: &str) -> StorageResult<String> {
        query_scalar(sql::role_group_query())
            .bind(user_id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub(super) async fn post_group(&self, user_id: &str) -> StorageResult<String> {
        query_scalar(sql::post_group_query())
            .bind(user_id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub(super) async fn dept_name(&self, user_id: &str) -> StorageResult<Option<String>> {
        query_scalar(sql::dept_name_query())
            .bind(user_id)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn post_ids(&self, user_id: &str) -> StorageResult<Vec<String>> {
        query_scalar("SELECT post_id FROM sys_user_post WHERE user_id = $1 ORDER BY post_id ASC")
            .bind(user_id)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn permissions(&self, user_id: &str) -> StorageResult<Vec<String>> {
        query_scalar(sql::permission_query())
            .bind(user_id)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }
}
