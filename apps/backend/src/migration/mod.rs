use std::{collections::HashMap, path::PathBuf};

use crate::BackendResult;
#[cfg(test)]
use sqlx::AssertSqlSafe;
use sqlx::{
    PgConnection, PgPool,
    migrate::{Migrate, MigrateError, Migrator},
    query,
};

mod readiness;
pub use readiness::ensure_runtime_schema_ready;

const MIGRATIONS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../migrations");
const MIGRATION_TABLE_NAME: &str = "_sqlx_migrations";
#[cfg(test)]
const MANAGED_FUNCTIONS: &[&str] = &["system_log_search_ngrams(TEXT)"];

const MANAGED_TABLES: &[&str] = &[
    "sys_installation_owner",
    "audit_outbox",
    "sys_user_session",
    "sys_logininfor",
    "sys_oper_log",
    "sys_system_log",
    "sys_notice_read",
    "sys_notice",
    "sys_job_execution",
    "sys_job",
    "sys_user_post",
    "sys_role_dept",
    "sys_role_menu",
    "sys_log_menu_hierarchy_role_grant",
    "sys_user_role",
    "sys_config",
    "sys_dict_data",
    "sys_dict_type",
    "sys_menu",
    "sys_post",
    "sys_user",
    "sys_role",
    "sys_dept",
];

#[derive(Debug)]
pub struct MigrationStatusRow {
    pub version: i64,
    pub kind: &'static str,
    pub description: String,
}

pub async fn up(pool: &PgPool, steps: Option<u32>) -> BackendResult<()> {
    let migrator = migrator().await?;
    if let Some(steps) = steps {
        apply_pending_steps(pool, steps, &migrator).await?;
        return Ok(());
    }
    run_migrator(pool, &migrator).await?;
    Ok(())
}

pub async fn reset_public_schema(pool: &PgPool) -> BackendResult<()> {
    let mut transaction = pool.begin().await?;
    query("DROP SCHEMA public CASCADE").execute(&mut *transaction).await?;
    query("CREATE SCHEMA public AUTHORIZATION CURRENT_USER").execute(&mut *transaction).await?;
    query("GRANT ALL ON SCHEMA public TO PUBLIC").execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(())
}

#[cfg(test)]
pub async fn down(pool: &PgPool, steps: Option<u32>) -> BackendResult<()> {
    let migrator = migrator().await?;
    let target = undo_target(pool, steps.unwrap_or(1), &migrator).await?;
    reject_forward_only_rollback(pool, &migrator, target).await?;
    undo_migrator(pool, &migrator, target).await?;
    Ok(())
}

pub async fn status(pool: &PgPool) -> BackendResult<Vec<MigrationStatusRow>> {
    let migrator = migrator().await?;
    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table(MIGRATION_TABLE_NAME).await?;
    let applied = conn.list_applied_migrations(MIGRATION_TABLE_NAME).await?;
    let applied_map = applied
        .into_iter()
        .map(|migration| (migration.version, migration.checksum.into_owned()))
        .collect::<HashMap<_, _>>();

    let mut rows = Vec::new();
    for migration in migrator.iter() {
        if migration.migration_type.is_down_migration() {
            continue;
        }
        let kind = match applied_map.get(&migration.version) {
            Some(checksum) if checksum.as_slice() == migration.checksum.as_ref() => "applied",
            Some(_) => "checksum_mismatch",
            None => "pending",
        };
        rows.push(MigrationStatusRow {
            version: migration.version,
            kind,
            description: migration.description.to_string(),
        });
    }

    for (version, _) in applied_map {
        if !migrator.version_exists(version) {
            rows.push(MigrationStatusRow {
                version,
                kind: "missing_local_file",
                description: "applied migration missing from local source".into(),
            });
        }
    }

    rows.sort_by_key(|row| row.version);
    Ok(rows)
}

#[cfg(test)]
pub async fn fresh(pool: &PgPool) -> BackendResult<()> {
    let migrator = migrator().await?;
    reset_database(pool).await?;
    run_migrator(pool, &migrator).await?;
    Ok(())
}

async fn migrator() -> Result<Migrator, MigrateError> {
    Migrator::new(PathBuf::from(MIGRATIONS_DIR)).await
}

async fn run_migrator(pool: &PgPool, migrator: &Migrator) -> Result<(), MigrateError> {
    let mut connection = pool.acquire().await?;
    let result = migrator.run_direct(None, &mut *connection, false).await;
    unlock_failed_migration(&mut connection, result).await
}

#[cfg(test)]
async fn undo_migrator(pool: &PgPool, migrator: &Migrator, target: i64) -> Result<(), MigrateError> {
    let mut connection = pool.acquire().await?;
    let result = migrator.undo(&mut *connection, target).await;
    unlock_failed_migration(&mut connection, result).await
}

async fn unlock_failed_migration(connection: &mut PgConnection, result: Result<(), MigrateError>) -> Result<(), MigrateError> {
    if result.is_err() {
        connection.unlock().await?;
    }
    result
}

async fn apply_pending_steps(pool: &PgPool, steps: u32, migrator: &Migrator) -> Result<(), MigrateError> {
    if steps == 0 {
        return Ok(());
    }

    let pending_versions = pending_up_versions(pool, migrator).await?;
    if pending_versions.is_empty() {
        return Ok(());
    }

    let max_index = pending_versions.len().min(steps as usize) - 1;
    let target_version = pending_versions[max_index];

    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table(MIGRATION_TABLE_NAME).await?;
    if conn.dirty_version(MIGRATION_TABLE_NAME).await?.is_some() {
        drop(conn);
        return run_migrator(pool, migrator).await;
    }

    let applied = conn.list_applied_migrations(MIGRATION_TABLE_NAME).await?;
    let applied_map = applied
        .into_iter()
        .map(|migration| (migration.version, migration.checksum.into_owned()))
        .collect::<HashMap<_, _>>();

    for migration in migrator.iter() {
        if migration.migration_type.is_down_migration() || migration.version > target_version {
            continue;
        }

        match applied_map.get(&migration.version) {
            Some(checksum) if checksum.as_slice() == migration.checksum.as_ref() => {}
            Some(_) => return Err(MigrateError::VersionMismatch(migration.version)),
            None => {
                conn.apply(MIGRATION_TABLE_NAME, migration).await?;
            }
        }
    }

    Ok(())
}

async fn pending_up_versions(pool: &PgPool, migrator: &Migrator) -> Result<Vec<i64>, MigrateError> {
    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table(MIGRATION_TABLE_NAME).await?;
    let applied = conn.list_applied_migrations(MIGRATION_TABLE_NAME).await?;
    let applied_versions = applied.into_iter().map(|migration| migration.version).collect::<std::collections::HashSet<_>>();
    Ok(migrator
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .filter(|migration| !applied_versions.contains(&migration.version))
        .map(|migration| migration.version)
        .collect())
}

#[cfg(test)]
async fn undo_target(pool: &PgPool, steps: u32, _migrator: &Migrator) -> Result<i64, MigrateError> {
    let applied = applied_up_versions(pool).await?;
    if applied.is_empty() {
        return Ok(0);
    }
    let steps = steps as usize;
    if steps >= applied.len() {
        return Ok(0);
    }
    Ok(applied[applied.len() - steps - 1])
}

#[cfg(test)]
async fn applied_up_versions(pool: &PgPool) -> Result<Vec<i64>, MigrateError> {
    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table(MIGRATION_TABLE_NAME).await?;
    Ok(conn
        .list_applied_migrations(MIGRATION_TABLE_NAME)
        .await?
        .into_iter()
        .map(|migration| migration.version)
        .collect())
}

#[cfg(test)]
async fn reject_forward_only_rollback(pool: &PgPool, migrator: &Migrator, target: i64) -> BackendResult<()> {
    let applied_versions = applied_up_versions(pool).await?;
    let forward_only = migrator
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .filter(|migration| !migration.migration_type.is_reversible())
        .find(|migration| migration.version > target && applied_versions.contains(&migration.version));

    if let Some(migration) = forward_only {
        return Err(std::io::Error::other(format!("cannot roll back through forward-only migration {}", migration.version)).into());
    }
    Ok(())
}

#[cfg(test)]
async fn reset_database(pool: &PgPool) -> BackendResult<()> {
    let mut tx = pool.begin().await?;
    for table in MANAGED_TABLES {
        query(AssertSqlSafe(format!("DROP TABLE IF EXISTS {table} CASCADE"))).execute(&mut *tx).await?;
    }
    for function in MANAGED_FUNCTIONS {
        query(AssertSqlSafe(format!("DROP FUNCTION IF EXISTS {function} CASCADE")))
            .execute(&mut *tx)
            .await?;
    }
    query("DROP TABLE IF EXISTS _sqlx_migrations").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests;
