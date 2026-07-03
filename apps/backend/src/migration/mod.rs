use std::collections::HashMap;

use crate::BackendResult;
use sqlx::{
    PgPool,
    migrate::{Migrate, MigrateError, Migrator},
    query,
};

mod readiness;
pub use readiness::ensure_runtime_schema_ready;

pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

const MANAGED_TABLES: &[&str] = &[
    "role_menu_permissions",
    "role_api_permissions",
    "menu_items",
    "menu_sections",
    "api_permissions",
    "roles",
    "users",
];

#[derive(Debug)]
pub struct MigrationStatusRow {
    pub version: i64,
    pub kind: &'static str,
    pub description: String,
}

pub async fn up(pool: &PgPool, steps: Option<u32>) -> BackendResult<()> {
    if let Some(steps) = steps {
        apply_pending_steps(pool, steps).await?;
        return Ok(());
    }
    MIGRATOR.run(pool).await?;
    Ok(())
}

pub async fn down(pool: &PgPool, steps: Option<u32>) -> BackendResult<()> {
    let target = undo_target(pool, steps.unwrap_or(1)).await?;
    MIGRATOR.undo(pool, target).await?;
    Ok(())
}

pub async fn status(pool: &PgPool) -> BackendResult<Vec<MigrationStatusRow>> {
    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table().await?;
    let applied = conn.list_applied_migrations().await?;
    let applied_map = applied
        .into_iter()
        .map(|migration| (migration.version, migration.checksum.into_owned()))
        .collect::<HashMap<_, _>>();

    let mut rows = Vec::new();
    for migration in MIGRATOR.iter() {
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
        if !MIGRATOR.version_exists(version) {
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

pub async fn fresh(pool: &PgPool) -> BackendResult<()> {
    reset_database(pool).await?;
    MIGRATOR.run(pool).await?;
    Ok(())
}

pub async fn refresh(pool: &PgPool) -> BackendResult<()> {
    reset(pool).await?;
    MIGRATOR.run(pool).await?;
    Ok(())
}

pub async fn reset(pool: &PgPool) -> BackendResult<()> {
    let count = applied_up_migration_count(pool).await?;
    if count == 0 {
        return Ok(());
    }
    MIGRATOR.undo(pool, 0).await?;
    Ok(())
}

async fn apply_pending_steps(pool: &PgPool, steps: u32) -> Result<(), MigrateError> {
    if steps == 0 {
        return Ok(());
    }

    let pending_versions = pending_up_versions(pool).await?;
    if pending_versions.is_empty() {
        return Ok(());
    }

    let max_index = pending_versions.len().min(steps as usize) - 1;
    let target_version = pending_versions[max_index];

    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table().await?;
    if conn.dirty_version().await?.is_some() {
        return MIGRATOR.run(pool).await;
    }

    let applied = conn.list_applied_migrations().await?;
    let applied_map = applied
        .into_iter()
        .map(|migration| (migration.version, migration.checksum.into_owned()))
        .collect::<HashMap<_, _>>();

    for migration in MIGRATOR.iter() {
        if migration.migration_type.is_down_migration() || migration.version > target_version {
            continue;
        }

        match applied_map.get(&migration.version) {
            Some(checksum) if checksum.as_slice() == migration.checksum.as_ref() => {}
            Some(_) => return Err(MigrateError::VersionMismatch(migration.version)),
            None => {
                conn.apply(migration).await?;
            }
        }
    }

    Ok(())
}

async fn pending_up_versions(pool: &PgPool) -> Result<Vec<i64>, MigrateError> {
    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table().await?;
    let applied = conn.list_applied_migrations().await?;
    let applied_versions = applied.into_iter().map(|migration| migration.version).collect::<std::collections::HashSet<_>>();
    Ok(MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .filter(|migration| !applied_versions.contains(&migration.version))
        .map(|migration| migration.version)
        .collect())
}

async fn undo_target(pool: &PgPool, steps: u32) -> Result<i64, MigrateError> {
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

async fn applied_up_versions(pool: &PgPool) -> Result<Vec<i64>, MigrateError> {
    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table().await?;
    Ok(conn.list_applied_migrations().await?.into_iter().map(|migration| migration.version).collect())
}

async fn applied_up_migration_count(pool: &PgPool) -> Result<usize, MigrateError> {
    Ok(applied_up_versions(pool).await?.len())
}

async fn reset_database(pool: &PgPool) -> BackendResult<()> {
    let mut tx = pool.begin().await?;
    for table in MANAGED_TABLES {
        query(&format!("DROP TABLE IF EXISTS {table} CASCADE")).execute(&mut *tx).await?;
    }
    query("DROP TABLE IF EXISTS _sqlx_migrations").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests;
