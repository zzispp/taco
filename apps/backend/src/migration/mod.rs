use std::collections::HashMap;

use sqlx::{
    PgPool,
    migrate::{Migrate, MigrateError, Migrator},
    query,
};

use crate::BackendResult;

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
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use sqlx::{PgPool, postgres::PgPoolOptions, query, query_scalar};

    use super::{down, fresh, refresh, reset, status, up};

    const TEST_DB_ADMIN_URL: &str = "postgres://postgres:123456@localhost:5433/postgres";
    const TEST_DB_URL_PREFIX: &str = "postgres://postgres:123456@localhost:5433";
    const MIGRATION_TOTAL: usize = 2;
    const USERS_TABLE_REGCLASS: &str = "public.users";

    #[tokio::test]
    async fn migrations_support_full_up_down_cycle() {
        let database = TestDatabase::create().await;
        let pool = database.pool();

        assert_status_counts(pool, 0, MIGRATION_TOTAL).await;

        up(pool, Some(1)).await.unwrap();
        assert_status_counts(pool, 1, MIGRATION_TOTAL - 1).await;
        assert!(users_table_exists(pool).await);

        up(pool, None).await.unwrap();
        assert_status_counts(pool, MIGRATION_TOTAL, 0).await;
        assert!(users_table_exists(pool).await);

        down(pool, Some(1)).await.unwrap();
        assert_status_counts(pool, 1, MIGRATION_TOTAL - 1).await;
        assert!(users_table_exists(pool).await);

        refresh(pool).await.unwrap();
        assert_status_counts(pool, MIGRATION_TOTAL, 0).await;
        assert!(users_table_exists(pool).await);

        reset(pool).await.unwrap();
        assert_status_counts(pool, 0, MIGRATION_TOTAL).await;
        assert!(!users_table_exists(pool).await);

        fresh(pool).await.unwrap();
        assert_status_counts(pool, MIGRATION_TOTAL, 0).await;
        assert!(users_table_exists(pool).await);

        database.drop().await;
    }

    struct TestDatabase {
        admin_pool: PgPool,
        pool: PgPool,
        name: String,
    }

    impl TestDatabase {
        async fn create() -> Self {
            let admin_pool = PgPoolOptions::new().max_connections(1).connect(TEST_DB_ADMIN_URL).await.unwrap();
            let name = test_database_name();

            query(&format!(r#"CREATE DATABASE "{name}""#)).execute(&admin_pool).await.unwrap();

            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&format!("{TEST_DB_URL_PREFIX}/{name}"))
                .await
                .unwrap();

            Self { admin_pool, pool, name }
        }

        fn pool(&self) -> &PgPool {
            &self.pool
        }

        async fn drop(self) {
            self.pool.close().await;
            query("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = $1 AND pid <> pg_backend_pid()")
                .bind(&self.name)
                .execute(&self.admin_pool)
                .await
                .unwrap();
            query(&format!(r#"DROP DATABASE IF EXISTS "{}""#, self.name))
                .execute(&self.admin_pool)
                .await
                .unwrap();
            self.admin_pool.close().await;
        }
    }

    async fn assert_status_counts(pool: &PgPool, applied: usize, pending: usize) {
        let rows = status(pool).await.unwrap();
        assert_eq!(rows.iter().filter(|row| row.kind == "applied").count(), applied);
        assert_eq!(rows.iter().filter(|row| row.kind == "pending").count(), pending);
    }

    async fn users_table_exists(pool: &PgPool) -> bool {
        query_scalar::<_, bool>("SELECT to_regclass($1) IS NOT NULL")
            .bind(USERS_TABLE_REGCLASS)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    fn test_database_name() -> String {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
        format!("hook_migration_test_{}_{}", std::process::id(), suffix)
    }
}
