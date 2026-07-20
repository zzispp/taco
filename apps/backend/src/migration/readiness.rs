use std::collections::HashSet;

use sqlx::{PgPool, migrate::Migrator, query_as, query_scalar};

use crate::BackendResult;

use super::{MANAGED_TABLES, migrator};

const MIGRATION_UP_COMMAND: &str = "taco --data-dir <path> --config-encryption-key <key> migration up";
const MIGRATIONS_TABLE: &str = "_sqlx_migrations";
const PUBLIC_SCHEMA: &str = "public";

pub async fn ensure_runtime_schema_ready(pool: &PgPool) -> BackendResult<()> {
    if let Some(version) = dirty_migration_version(pool).await? {
        return Err(dirty_schema_error(version).into());
    }

    let migrator = migrator().await?;
    validate_applied_migration_sources(pool, &migrator).await?;

    let pending_versions = pending_migration_versions(pool, &migrator).await?;
    if !pending_versions.is_empty() {
        let versions = pending_versions.join(", ");
        return Err(format!("database schema is not ready: pending migrations [{versions}]. Run `{MIGRATION_UP_COMMAND}` before starting backend.").into());
    }

    let missing_tables = missing_managed_tables(pool).await?;
    if missing_tables.is_empty() {
        return Ok(());
    }

    let tables = missing_tables.join(", ");
    Err(format!("database schema is incomplete: missing managed tables [{tables}]; repair the PostgreSQL schema before starting Taco.").into())
}

async fn validate_applied_migration_sources(pool: &PgPool, migrator: &Migrator) -> BackendResult<()> {
    if !managed_table_exists(pool, MIGRATIONS_TABLE).await? {
        return Ok(());
    }

    let rows = query_as::<_, (i64, Vec<u8>)>("SELECT version, checksum FROM _sqlx_migrations WHERE success = TRUE ORDER BY version")
        .fetch_all(pool)
        .await?;

    for (version, checksum) in rows {
        let migration = migrator.iter().find(|item| item.version == version && item.migration_type.is_up_migration());
        match migration {
            Some(item) if item.checksum.as_ref() == checksum.as_slice() => {}
            Some(_) => return Err(format!("database schema migration checksum mismatch at version {version}").into()),
            None => return Err(format!("database schema contains applied migration {version} but the local migration file is missing").into()),
        }
    }

    Ok(())
}

async fn pending_migration_versions(pool: &PgPool, migrator: &Migrator) -> BackendResult<Vec<String>> {
    let applied_versions = applied_migration_versions(pool).await?;
    Ok(migrator
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

fn dirty_schema_error(version: i64) -> String {
    format!("database schema is dirty at migration {version}; repair the migration state before starting Taco.")
}
