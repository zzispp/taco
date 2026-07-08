use std::collections::HashMap;

use sqlx::{AssertSqlSafe, Postgres, query, query_as, query_scalar};
use storage::{StorageError, StorageResult};

use crate::domain::RoleDataScopeInput;
use types::rbac::DATA_SCOPE_CUSTOM;

const ROLE_MENU_TABLE: &str = "sys_role_menu";
const ROLE_DEPT_TABLE: &str = "sys_role_dept";
const MENU_ID_COLUMN: &str = "menu_id";
const DEPT_ID_COLUMN: &str = "dept_id";

type ParentById = HashMap<String, String>;
type RoleTx<'a> = sqlx::Transaction<'a, Postgres>;

struct RelationTarget<'a> {
    table: &'a str,
    column: &'a str,
    role_id: &'a str,
}

struct RelationReplacement<'a> {
    target: RelationTarget<'a>,
    ids: Vec<String>,
}

struct RoleUniqueCheck<'a> {
    column: &'a str,
    value: &'a str,
    current_id: Option<&'a str>,
}

pub(super) async fn normalize_data_scope_dept_ids(pool: &sqlx::PgPool, input: &RoleDataScopeInput) -> StorageResult<Vec<String>> {
    if input.data_scope != DATA_SCOPE_CUSTOM || !input.dept_check_strictly {
        return Ok(unique_ids(&input.dept_ids));
    }
    let rows = query_as::<_, (String, String)>("SELECT dept_id, ancestors FROM sys_dept WHERE dept_id = ANY($1) AND del_flag = '0'")
        .bind(&input.dept_ids)
        .fetch_all(pool)
        .await?;
    let mut ids = Vec::new();
    for (_, ancestors) in rows {
        for ancestor in ancestors.split(',').filter(|id| !id.is_empty() && *id != "0") {
            push_unique(&mut ids, ancestor.to_owned());
        }
    }
    for id in &input.dept_ids {
        push_unique(&mut ids, id.clone());
    }
    Ok(ids)
}

pub(super) async fn normalize_menu_ids(pool: &sqlx::PgPool, menu_ids: &[String]) -> StorageResult<Vec<String>> {
    let rows = query_as::<_, (String, String)>("SELECT menu_id, parent_id FROM sys_menu")
        .fetch_all(pool)
        .await?;
    let parent_by_id = rows.into_iter().collect::<ParentById>();
    let mut ids = Vec::new();
    for id in menu_ids {
        for parent_id in collect_menu_ancestors(&parent_by_id, id) {
            push_unique(&mut ids, parent_id);
        }
        push_unique(&mut ids, id.clone());
    }
    Ok(ids)
}

pub(super) async fn replace_menu_ids(pool: &sqlx::PgPool, role_id: &str, ids: Vec<String>) -> StorageResult<()> {
    replace_ids(
        pool,
        RelationReplacement {
            target: menu_target(role_id),
            ids,
        },
    )
    .await
}

pub(super) async fn replace_dept_ids(pool: &sqlx::PgPool, role_id: &str, ids: Vec<String>) -> StorageResult<()> {
    replace_ids(
        pool,
        RelationReplacement {
            target: dept_target(role_id),
            ids,
        },
    )
    .await
}

pub(super) async fn insert_dept_ids(tx: &mut RoleTx<'_>, role_id: &str, ids: Vec<String>) -> StorageResult<()> {
    insert_ids(
        tx,
        RelationReplacement {
            target: dept_target(role_id),
            ids,
        },
    )
    .await
}

pub(super) async fn menu_ids(pool: &sqlx::PgPool, role_id: &str) -> StorageResult<Vec<String>> {
    binding_ids(pool, menu_target(role_id)).await
}

pub(super) async fn dept_ids(pool: &sqlx::PgPool, role_id: &str) -> StorageResult<Vec<String>> {
    binding_ids(pool, dept_target(role_id)).await
}

pub(super) async fn role_name_exists(pool: &sqlx::PgPool, name: &str, current_id: Option<&str>) -> StorageResult<bool> {
    unique_exists(
        pool,
        RoleUniqueCheck {
            column: "role_name",
            value: name,
            current_id,
        },
    )
    .await
}

pub(super) async fn role_key_exists(pool: &sqlx::PgPool, key: &str, current_id: Option<&str>) -> StorageResult<bool> {
    unique_exists(
        pool,
        RoleUniqueCheck {
            column: "role_key",
            value: key,
            current_id,
        },
    )
    .await
}

pub(super) async fn replace_user_ids(pool: &sqlx::PgPool, role_id: &str, user_ids: Vec<String>) -> StorageResult<()> {
    let mut tx = pool.begin().await?;
    for id in user_ids {
        query("INSERT INTO sys_user_role (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(id)
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await.map_err(StorageError::from)
}

async fn replace_ids(pool: &sqlx::PgPool, replacement: RelationReplacement<'_>) -> StorageResult<()> {
    let mut tx = pool.begin().await?;
    query(AssertSqlSafe(format!("DELETE FROM {} WHERE role_id = $1", replacement.target.table)))
        .bind(replacement.target.role_id)
        .execute(&mut *tx)
        .await?;
    insert_ids(&mut tx, replacement).await?;
    tx.commit().await.map_err(StorageError::from)
}

async fn insert_ids(tx: &mut RoleTx<'_>, replacement: RelationReplacement<'_>) -> StorageResult<()> {
    for id in replacement.ids {
        query(AssertSqlSafe(format!(
            "INSERT INTO {} (role_id, {}) VALUES ($1, $2)",
            replacement.target.table, replacement.target.column
        )))
        .bind(replacement.target.role_id)
        .bind(id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn binding_ids(pool: &sqlx::PgPool, target: RelationTarget<'_>) -> StorageResult<Vec<String>> {
    query_scalar::<_, String>(AssertSqlSafe(format!(
        "SELECT {} FROM {} WHERE role_id = $1 ORDER BY {} ASC",
        target.column, target.table, target.column
    )))
    .bind(target.role_id)
    .fetch_all(pool)
    .await
    .map_err(StorageError::from)
}

async fn unique_exists(pool: &sqlx::PgPool, check: RoleUniqueCheck<'_>) -> StorageResult<bool> {
    query_scalar::<_, bool>(AssertSqlSafe(format!(
        "SELECT EXISTS(SELECT 1 FROM sys_role WHERE del_flag='0' AND {}=$1 AND ($2::text IS NULL OR role_id<>$2))",
        check.column
    )))
    .bind(check.value)
    .bind(check.current_id)
    .fetch_one(pool)
    .await
    .map_err(StorageError::from)
}

fn collect_menu_ancestors(parent_by_id: &ParentById, id: &str) -> Vec<String> {
    let Some(parent_id) = parent_by_id.get(id) else { return Vec::new() };
    if parent_id == "0" || !parent_by_id.contains_key(parent_id) {
        return Vec::new();
    }
    let mut ancestors = collect_menu_ancestors(parent_by_id, parent_id);
    ancestors.push(parent_id.clone());
    ancestors
}

fn unique_ids(ids: &[String]) -> Vec<String> {
    let mut unique = Vec::new();
    for id in ids {
        push_unique(&mut unique, id.clone());
    }
    unique
}

fn push_unique(ids: &mut Vec<String>, id: String) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn menu_target(role_id: &str) -> RelationTarget<'_> {
    RelationTarget {
        table: ROLE_MENU_TABLE,
        column: MENU_ID_COLUMN,
        role_id,
    }
}

fn dept_target(role_id: &str) -> RelationTarget<'_> {
    RelationTarget {
        table: ROLE_DEPT_TABLE,
        column: DEPT_ID_COLUMN,
        role_id,
    }
}
