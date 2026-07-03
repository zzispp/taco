use std::collections::HashSet;

use sqlx::{PgPool, query_scalar};

use crate::BackendResult;

use super::{MANAGED_TABLES, MIGRATOR};

const MIGRATION_UP_COMMAND: &str = "cargo run -p backend -- migration up";
const MIGRATION_REFRESH_COMMAND: &str = "cargo run -p backend -- migration refresh";
const MIGRATIONS_TABLE: &str = "_sqlx_migrations";
const PUBLIC_SCHEMA: &str = "public";

pub async fn ensure_runtime_schema_ready(pool: &PgPool) -> BackendResult<()> {
    let pending_versions = pending_migration_versions(pool).await?;
    if !pending_versions.is_empty() {
        let versions = pending_versions.join(", ");
        return Err(format!("database schema is not ready: pending migrations [{versions}]. Run `{MIGRATION_UP_COMMAND}` before starting backend.").into());
    }

    if let Some(version) = dirty_migration_version(pool).await? {
        return Err(format!(
            "database schema is dirty at migration {version}. Run `{MIGRATION_REFRESH_COMMAND}` or repair the migration state before starting backend."
        )
        .into());
    }

    let missing_tables = missing_managed_tables(pool).await?;
    if missing_tables.is_empty() {
        return Ok(());
    }

    let tables = missing_tables.join(", ");
    Err(format!("database schema is incomplete: missing managed tables [{tables}]. Run `{MIGRATION_REFRESH_COMMAND}` before starting backend.").into())
}

async fn pending_migration_versions(pool: &PgPool) -> BackendResult<Vec<String>> {
    let applied_versions = applied_migration_versions(pool).await?;
    Ok(MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .filter(|migration| !applied_versions.contains(&migration.version))
        .map(|migration| migration.version.to_string())
        .collect())
}

async fn applied_migration_versions(pool: &PgPool) -> BackendResult<HashSet<i64>> {
    if !managed_table_exists(pool, MIGRATIONS_TABLE).await? {
        return Ok(HashSet::new());
    }

    let versions = query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations WHERE success = TRUE")
        .fetch_all(pool)
        .await?;
    Ok(versions.into_iter().collect())
}

async fn dirty_migration_version(pool: &PgPool) -> BackendResult<Option<i64>> {
    if !managed_table_exists(pool, MIGRATIONS_TABLE).await? {
        return Ok(None);
    }

    query_scalar::<_, i64>("SELECT version FROM _sqlx_migrations WHERE success = FALSE ORDER BY version DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

async fn missing_managed_tables(pool: &PgPool) -> BackendResult<Vec<String>> {
    let mut missing_tables = Vec::new();
    for table in MANAGED_TABLES {
        if !managed_table_exists(pool, table).await? {
            missing_tables.push((*table).into());
        }
    }
    Ok(missing_tables)
}

async fn managed_table_exists(pool: &PgPool, table: &str) -> BackendResult<bool> {
    query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = $1 AND table_name = $2)")
        .bind(PUBLIC_SCHEMA)
        .bind(table)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}
