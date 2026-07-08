use kernel::pagination::Page;
use sqlx::{AssertSqlSafe, query_as, query_scalar};
use storage::{Database, StorageResult, database::to_i64};

use crate::{
    application::RoleListFilter,
    domain::{DataScopeFilter, Role},
    infra::records::RoleRecord,
};

use super::sql::{role_page, role_page_sql, role_scoped_page_sql, role_scoped_total_sql, role_total_sql};

pub(super) async fn page(database: &Database, filter: RoleListFilter) -> StorageResult<Page<Role>> {
    let total = query_scalar::<_, i64>(AssertSqlSafe(role_total_sql()))
        .bind(&filter.role_name)
        .bind(&filter.role_key)
        .bind(&filter.status)
        .bind(filter.system)
        .bind(&filter.begin_time)
        .bind(&filter.end_time)
        .fetch_one(database.pool())
        .await?;
    let items = query_as::<_, RoleRecord>(AssertSqlSafe(role_page_sql()))
        .bind(&filter.role_name)
        .bind(&filter.role_key)
        .bind(&filter.status)
        .bind(filter.system)
        .bind(&filter.begin_time)
        .bind(&filter.end_time)
        .bind(to_i64(filter.page.page_size)?)
        .bind(to_i64((filter.page.page - 1) * filter.page.page_size)?)
        .fetch_all(database.pool())
        .await?;
    role_page(items, total, filter)
}

pub(super) async fn page_scoped(database: &Database, filter: RoleListFilter, scope: DataScopeFilter) -> StorageResult<Page<Role>> {
    let total = query_scalar::<_, i64>(AssertSqlSafe(role_scoped_total_sql()))
        .bind(&filter.role_name)
        .bind(&filter.role_key)
        .bind(&filter.status)
        .bind(filter.system)
        .bind(&filter.begin_time)
        .bind(&filter.end_time)
        .bind(&scope.data_scope)
        .bind(&scope.user_id)
        .bind(&scope.dept_id)
        .bind(&scope.dept_ids)
        .fetch_one(database.pool())
        .await?;
    let items = query_as::<_, RoleRecord>(AssertSqlSafe(role_scoped_page_sql()))
        .bind(&filter.role_name)
        .bind(&filter.role_key)
        .bind(&filter.status)
        .bind(filter.system)
        .bind(&filter.begin_time)
        .bind(&filter.end_time)
        .bind(&scope.data_scope)
        .bind(&scope.user_id)
        .bind(&scope.dept_id)
        .bind(&scope.dept_ids)
        .bind(to_i64(filter.page.page_size)?)
        .bind(to_i64((filter.page.page - 1) * filter.page.page_size)?)
        .fetch_all(database.pool())
        .await?;
    role_page(items, total, filter)
}
