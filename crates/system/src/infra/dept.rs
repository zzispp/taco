use kernel::pagination::Page;
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;

use crate::{
    application::DeptListFilter,
    domain::{Dept, DeptInput},
};
use types::rbac::DataScopeFilter;

use super::{dept_sql, mapping::dept, page, record::DeptRecord};

pub(super) const COLUMNS: &str = "dept_id,parent_id,ancestors,dept_name,order_num,leader,phone,email,status,create_time::text AS create_time";

#[derive(Clone)]
pub struct DeptQueries {
    database: Database,
}

impl DeptQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn page(&self, filter: DeptListFilter) -> StorageResult<Page<Dept>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(dept_sql::total_sql()))
            .bind(&filter.dept_name)
            .bind(&filter.leader)
            .bind(&filter.phone)
            .bind(&filter.email)
            .bind(&filter.status)
            .bind(filter.begin_time)
            .bind(filter.end_time)
            .fetch_one(self.database.pool())
            .await?;
        let items = query_as::<_, DeptRecord>(AssertSqlSafe(dept_sql::page_sql()))
            .bind(&filter.dept_name)
            .bind(&filter.leader)
            .bind(&filter.phone)
            .bind(&filter.email)
            .bind(&filter.status)
            .bind(filter.begin_time)
            .bind(filter.end_time)
            .bind(page::limit(filter.page)?)
            .bind(page::offset(filter.page)?)
            .fetch_all(self.database.pool())
            .await?;
        page::page(items.into_iter().map(dept).collect(), total, filter.page)
    }

    pub async fn page_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> StorageResult<Page<Dept>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(dept_sql::scoped_total_sql()))
            .bind(&filter.dept_name)
            .bind(&filter.leader)
            .bind(&filter.phone)
            .bind(&filter.email)
            .bind(&filter.status)
            .bind(filter.begin_time)
            .bind(filter.end_time)
            .bind(&scope.data_scope)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .fetch_one(self.database.pool())
            .await?;
        let items = query_as::<_, DeptRecord>(AssertSqlSafe(dept_sql::scoped_page_sql()))
            .bind(&filter.dept_name)
            .bind(&filter.leader)
            .bind(&filter.phone)
            .bind(&filter.email)
            .bind(&filter.status)
            .bind(filter.begin_time)
            .bind(filter.end_time)
            .bind(&scope.data_scope)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .bind(page::limit(filter.page)?)
            .bind(page::offset(filter.page)?)
            .fetch_all(self.database.pool())
            .await?;
        page::page(items.into_iter().map(dept).collect(), total, filter.page)
    }

    pub async fn list(&self, filter: DeptListFilter) -> StorageResult<Vec<Dept>> {
        query_as::<_, DeptRecord>(AssertSqlSafe(dept_sql::list_sql()))
            .bind(&filter.dept_name)
            .bind(&filter.leader)
            .bind(&filter.phone)
            .bind(&filter.email)
            .bind(&filter.status)
            .bind(filter.begin_time)
            .bind(filter.end_time)
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(dept).collect())
            .map_err(StorageError::from)
    }

    pub async fn list_scoped(&self, filter: DeptListFilter, scope: DataScopeFilter) -> StorageResult<Vec<Dept>> {
        query_as::<_, DeptRecord>(AssertSqlSafe(dept_sql::scoped_list_sql()))
            .bind(&filter.dept_name)
            .bind(&filter.leader)
            .bind(&filter.phone)
            .bind(&filter.email)
            .bind(&filter.status)
            .bind(filter.begin_time)
            .bind(filter.end_time)
            .bind(&scope.data_scope)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(dept).collect())
            .map_err(StorageError::from)
    }

    pub async fn list_excluding(&self, id: &str) -> StorageResult<Vec<Dept>> {
        query_as::<_, DeptRecord>(AssertSqlSafe(format!("SELECT {COLUMNS} FROM sys_dept WHERE del_flag='0' AND dept_id<>$1 AND (',' || ancestors || ',') NOT LIKE '%,' || $1 || ',%' ORDER BY parent_id ASC, order_num ASC")))
            .bind(id)
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(dept).collect())
            .map_err(StorageError::from)
    }

    pub async fn create(&self, input: DeptInput) -> StorageResult<Dept> {
        let id = self.database.next_id();
        query(dept_sql::insert_sql())
            .bind(&id)
            .bind(&input.parent_id)
            .bind(ancestors(self.database.pool(), &input.parent_id).await?)
            .bind(input.dept_name)
            .bind(input.order_num)
            .bind(input.leader)
            .bind(input.phone)
            .bind(input.email)
            .bind(input.status)
            .bind(OffsetDateTime::now_utc())
            .execute(self.database.pool())
            .await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace(&self, id: &str, input: DeptInput) -> StorageResult<Dept> {
        let current = self.find(id).await?.ok_or(StorageError::NotFound)?;
        let next_ancestors = ancestors(self.database.pool(), &input.parent_id).await?;
        let old_prefix = current.ancestors.clone();
        let new_prefix = next_ancestors.clone();
        let result = query(dept_sql::update_sql())
            .bind(id)
            .bind(&input.parent_id)
            .bind(&next_ancestors)
            .bind(input.dept_name)
            .bind(input.order_num)
            .bind(input.leader)
            .bind(input.phone)
            .bind(input.email)
            .bind(input.status)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())?;
        if current.parent_id != input.parent_id {
            update_child_ancestors(
                self.database.pool(),
                ChildAncestorsUpdate {
                    id,
                    old_prefix: &old_prefix,
                    new_prefix: &new_prefix,
                },
            )
            .await?;
        }
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_sort(&self, id: &str, order_num: i64) -> StorageResult<Dept> {
        let result = query("UPDATE sys_dept SET order_num=$2,update_time=CURRENT_TIMESTAMP WHERE dept_id=$1 AND del_flag='0'")
            .bind(id)
            .bind(order_num)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: &str) -> StorageResult<()> {
        let result = query("UPDATE sys_dept SET del_flag = '2', update_time = CURRENT_TIMESTAMP WHERE dept_id = $1 AND del_flag = '0'")
            .bind(id)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())
    }

    pub async fn has_children(&self, id: &str) -> StorageResult<bool> {
        exists(
            self.database.pool(),
            "SELECT EXISTS(SELECT 1 FROM sys_dept WHERE parent_id = $1 AND del_flag = '0')",
            id,
        )
        .await
    }

    pub async fn has_users(&self, id: &str) -> StorageResult<bool> {
        exists(
            self.database.pool(),
            "SELECT EXISTS(SELECT 1 FROM sys_user WHERE dept_id = $1 AND del_flag = '0')",
            id,
        )
        .await
    }

    pub async fn has_normal_children(&self, id: &str) -> StorageResult<bool> {
        exists(
            self.database.pool(),
            "SELECT EXISTS(SELECT 1 FROM sys_dept WHERE parent_id = $1 AND del_flag = '0' AND status = '0')",
            id,
        )
        .await
    }

    pub async fn find(&self, id: &str) -> StorageResult<Option<Dept>> {
        query_as::<_, DeptRecord>(AssertSqlSafe(format!("SELECT {COLUMNS} FROM sys_dept WHERE dept_id = $1 AND del_flag = '0'")))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map(|record| record.map(dept))
            .map_err(StorageError::from)
    }
}

async fn ancestors(pool: &sqlx::PgPool, parent_id: &str) -> StorageResult<String> {
    if parent_id == "0" {
        return Ok("0".into());
    }
    let parent = query_scalar::<_, String>("SELECT ancestors FROM sys_dept WHERE dept_id = $1 AND del_flag = '0'")
        .bind(parent_id)
        .fetch_optional(pool)
        .await?
        .ok_or(StorageError::NotFound)?;
    Ok(format!("{parent},{parent_id}"))
}

async fn exists(pool: &sqlx::PgPool, sql: &'static str, id: &str) -> StorageResult<bool> {
    query_scalar::<_, bool>(sql).bind(id).fetch_one(pool).await.map_err(StorageError::from)
}

struct ChildAncestorsUpdate<'a> {
    id: &'a str,
    old_prefix: &'a str,
    new_prefix: &'a str,
}

async fn update_child_ancestors(pool: &sqlx::PgPool, update: ChildAncestorsUpdate<'_>) -> StorageResult<()> {
    query(
        r#"
        UPDATE sys_dept
        SET ancestors = $3 || substring(ancestors from length($2) + 1),
            update_time = CURRENT_TIMESTAMP
        WHERE del_flag = '0' AND (',' || ancestors || ',') LIKE '%,' || $1 || ',%'
        "#,
    )
    .bind(update.id)
    .bind(format!("{},{}", update.old_prefix, update.id))
    .bind(format!("{},{}", update.new_prefix, update.id))
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(StorageError::from)
}

fn ensure_rows(rows: u64) -> StorageResult<()> {
    if rows == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}
