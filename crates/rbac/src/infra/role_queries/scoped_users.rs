use sqlx::query_scalar;
use storage::{StorageError, StorageResult};

use crate::domain::DataScopeFilter;

use super::{RoleQueries, sql::scoped_user_ids_sql};

impl RoleQueries {
    pub async fn scoped_user_ids(&self, user_ids: &[String], scope: DataScopeFilter) -> StorageResult<Vec<String>> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }
        query_scalar(scoped_user_ids_sql())
            .bind(user_ids)
            .bind(&scope.data_scope)
            .bind(&scope.user_id)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }
}
