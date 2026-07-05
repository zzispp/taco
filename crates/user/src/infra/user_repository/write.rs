use sqlx::{Postgres, QueryBuilder, query, query_scalar};
use storage::{StorageError, StorageResult};
use types::user::UserId;

use crate::application::ReplaceUserRecord;

use super::sql;

pub async fn execute_user_update(tx: &mut sqlx::Transaction<'_, Postgres>, id: &UserId, input: &ReplaceUserRecord) -> StorageResult<()> {
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

pub async fn replace_relations(tx: &mut sqlx::Transaction<'_, Postgres>, user_id: &str, role_ids: Vec<String>, post_ids: Vec<String>) -> StorageResult<()> {
    query("DELETE FROM sys_user_role WHERE user_id = $1").bind(user_id).execute(&mut **tx).await?;
    query("DELETE FROM sys_user_post WHERE user_id = $1").bind(user_id).execute(&mut **tx).await?;
    insert_ids(tx, "sys_user_role", "role_id", user_id, role_ids).await?;
    insert_ids(tx, "sys_user_post", "post_id", user_id, post_ids).await
}

pub async fn replace_roles(tx: &mut sqlx::Transaction<'_, Postgres>, user_id: &str, role_ids: Vec<String>) -> StorageResult<()> {
    query("DELETE FROM sys_user_role WHERE user_id = $1").bind(user_id).execute(&mut **tx).await?;
    insert_ids(tx, "sys_user_role", "role_id", user_id, role_ids).await
}

pub async fn ensure_dept_exists(pool: &sqlx::PgPool, dept_id: Option<&str>) -> StorageResult<()> {
    match dept_id {
        Some(id) => ensure_ids_exist(pool, "sys_dept", "dept_id", &[id.into()]).await,
        None => Ok(()),
    }
}

pub async fn ensure_ids_exist(pool: &sqlx::PgPool, table: &str, column: &str, ids: &[String]) -> StorageResult<()> {
    for id in ids {
        let sql = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE {column} = $1)");
        if !query_scalar::<_, bool>(&sql).bind(id).fetch_one(pool).await? {
            return Err(StorageError::Conflict(format!("{table}.{column} does not exist: {id}")));
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

async fn insert_ids(tx: &mut sqlx::Transaction<'_, Postgres>, table: &str, id_column: &str, user_id: &str, ids: Vec<String>) -> StorageResult<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let mut builder = QueryBuilder::<Postgres>::new(format!("INSERT INTO {table} (user_id, {id_column}) "));
    builder.push_values(ids, |mut row, id| {
        row.push_bind(user_id).push_bind(id);
    });
    builder.build().execute(&mut **tx).await?;
    Ok(())
}
