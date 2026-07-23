use crate::{
    domain::User,
    infra::user_repository::{mapping::user, record::UserRecord, sql},
};
use rbac::domain::DataScopeFilter;
use sqlx::{AssertSqlSafe, query_as, query_scalar};
use storage::{StorageError, StorageResult};

use super::{UserQueries, relations};

impl UserQueries {
    pub(super) async fn users(&self, records: Vec<UserRecord>) -> StorageResult<Vec<User>> {
        let ids = records.iter().map(|record| record.user_id.clone()).collect::<Vec<_>>();
        let mut relations = relations::load(&self.database, &ids).await?;
        records
            .into_iter()
            .map(|record| {
                let relation = relations::take(&mut relations, &record.user_id)?;
                user(record, relation)
            })
            .collect()
    }
    pub async fn scoped_existing_user_ids(&self, ids: Vec<String>, scope: &DataScopeFilter) -> StorageResult<Vec<String>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        query_scalar(sql::scoped_existing_user_ids())
            .bind(scope.data_scope.code())
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
        let mut users = self.users(vec![record]).await?;
        Ok(users.pop().expect("one user record must map to one user"))
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
}
