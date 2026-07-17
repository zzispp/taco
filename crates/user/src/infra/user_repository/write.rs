use sqlx::{AssertSqlSafe, Postgres, QueryBuilder, query, query_scalar};
use storage::{ObservedPgPool, StorageError, StorageResult};
use types::user::UserId;

use crate::application::ReplaceUserRecord;

use super::sql;

type UserTx<'a> = sqlx::Transaction<'a, Postgres>;

pub struct UserRelationIds {
    pub role_ids: Vec<String>,
    pub post_ids: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct ReferenceTable {
    table: &'static str,
    column: &'static str,
}

struct RelationInsert<'a> {
    table: &'a str,
    column: &'a str,
    user_id: &'a str,
    ids: Vec<String>,
}

impl ReferenceTable {
    const fn dept() -> Self {
        Self {
            table: "sys_dept",
            column: "dept_id",
        }
    }

    pub const fn role() -> Self {
        Self {
            table: "sys_role",
            column: "role_id",
        }
    }

    pub const fn post() -> Self {
        Self {
            table: "sys_post",
            column: "post_id",
        }
    }
}

pub async fn execute_user_update(tx: &mut UserTx<'_>, id: &UserId, input: &ReplaceUserRecord) -> StorageResult<()> {
    let result = if let Some(password_hash) = &input.password_hash {
        bind_user_update(query(sql::update_with_password()), id, input)
            .bind(password_hash)
            .execute(&mut **tx)
            .await?
    } else {
        bind_user_update(query(sql::update_without_password()), id, input).execute(&mut **tx).await?
    };
    ensure_rows_affected(result.rows_affected())
}

fn bind_user_update<'q>(
    query: sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>,
    id: &'q UserId,
    input: &'q ReplaceUserRecord,
) -> sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments> {
    query
        .bind(&id.0)
        .bind(&input.dept_id)
        .bind(&input.username)
        .bind(&input.nick_name)
        .bind(&input.email)
        .bind(&input.phonenumber)
        .bind(&input.sex)
        .bind(&input.status)
        .bind(&input.remark)
}

pub async fn replace_relations(tx: &mut UserTx<'_>, user_id: &str, ids: UserRelationIds) -> StorageResult<()> {
    query("DELETE FROM sys_user_role WHERE user_id = $1").bind(user_id).execute(&mut **tx).await?;
    query("DELETE FROM sys_user_post WHERE user_id = $1").bind(user_id).execute(&mut **tx).await?;
    insert_ids(
        tx,
        RelationInsert {
            table: "sys_user_role",
            column: "role_id",
            user_id,
            ids: ids.role_ids,
        },
    )
    .await?;
    insert_ids(
        tx,
        RelationInsert {
            table: "sys_user_post",
            column: "post_id",
            user_id,
            ids: ids.post_ids,
        },
    )
    .await
}

pub async fn replace_roles(tx: &mut UserTx<'_>, user_id: &str, role_ids: Vec<String>) -> StorageResult<()> {
    query("DELETE FROM sys_user_role WHERE user_id = $1").bind(user_id).execute(&mut **tx).await?;
    insert_ids(
        tx,
        RelationInsert {
            table: "sys_user_role",
            column: "role_id",
            user_id,
            ids: role_ids,
        },
    )
    .await
}

pub async fn ensure_dept_exists(pool: ObservedPgPool, dept_id: Option<&str>) -> StorageResult<()> {
    match dept_id {
        Some(id) => ensure_ids_exist(pool, ReferenceTable::dept(), &[id.into()]).await,
        None => Ok(()),
    }
}

pub async fn ensure_ids_exist(pool: ObservedPgPool, reference: ReferenceTable, ids: &[String]) -> StorageResult<()> {
    for id in ids {
        let sql = format!("SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)", reference.table, reference.column);
        if !query_scalar::<_, bool>(AssertSqlSafe(sql)).bind(id).fetch_one(pool.clone()).await? {
            return Err(StorageError::Conflict(format!("{}.{} does not exist: {id}", reference.table, reference.column)));
        }
    }
    Ok(())
}

pub fn ensure_rows_affected(rows_affected: u64) -> StorageResult<()> {
    if rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

async fn insert_ids(tx: &mut UserTx<'_>, input: RelationInsert<'_>) -> StorageResult<()> {
    if input.ids.is_empty() {
        return Ok(());
    }
    let mut builder = QueryBuilder::<Postgres>::new(format!("INSERT INTO {} (user_id, {}) ", input.table, input.column));
    builder.push_values(input.ids, |mut row, id| {
        row.push_bind(input.user_id).push_bind(id);
    });
    builder.build().execute(&mut **tx).await?;
    Ok(())
}
