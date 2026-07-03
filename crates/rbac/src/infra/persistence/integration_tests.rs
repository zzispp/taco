use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::{Executor, PgPool, postgres::PgPoolOptions, query};
use storage::{Database, connect_database};

use super::{ApiPermissionRecordInput, RbacStore, RoleApiBindingRecordInput, RoleRecordInput};

const TEST_DB_ADMIN_URL: &str = "postgres://postgres:123456@localhost:5433/postgres";
const TEST_DB_URL_PREFIX: &str = "postgres://postgres:123456@localhost:5433";

static NEXT_TEST_DB_ID: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn replace_role_apis_replaces_bindings_transactionally() {
    let database = TestDatabase::create().await;
    let rbac = RbacStore::new(database.database().clone());

    let api = rbac
        .create_api(ApiPermissionRecordInput {
            code: "test_api".into(),
            method: "GET".into(),
            path_pattern: "/api/test".into(),
            name: "Test".into(),
            group: "Tests".into(),
            enabled: true,
            system: false,
        })
        .await
        .unwrap();

    rbac.create_role(RoleRecordInput {
        code: "tester".into(),
        name: "Tester".into(),
        description: "tester".into(),
        enabled: true,
        system: false,
        sort_order: 1,
    })
    .await
    .unwrap();

    rbac.replace_role_apis(
        "tester",
        vec![RoleApiBindingRecordInput {
            role_code: "tester".into(),
            api_permission_id: api.id.clone(),
        }],
    )
    .await
    .unwrap();

    assert_eq!(rbac.role_api_ids("tester").await.unwrap(), vec![api.id.clone()]);

    rbac.replace_role_apis("tester", vec![]).await.unwrap();

    assert!(rbac.role_api_ids("tester").await.unwrap().is_empty());
    assert!(!rbac.role_has_api_bindings("tester").await.unwrap());

    database.drop().await;
}

struct TestDatabase {
    admin_pool: PgPool,
    database: Database,
    name: String,
}

impl TestDatabase {
    async fn create() -> Self {
        let admin_pool = PgPoolOptions::new().max_connections(1).connect(TEST_DB_ADMIN_URL).await.unwrap();
        let name = test_database_name();

        query(&format!(r#"CREATE DATABASE "{name}""#)).execute(&admin_pool).await.unwrap();

        let database = connect_database(&format!("{TEST_DB_URL_PREFIX}/{name}")).await.unwrap();
        for sql in migration_sqls() {
            database.pool().execute(sql).await.unwrap();
        }

        Self { admin_pool, database, name }
    }

    fn database(&self) -> &Database {
        &self.database
    }

    async fn drop(self) {
        self.database.pool().close().await;
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

fn migration_sqls() -> [&'static str; 2] {
    [
        include_str!("../../../../../migrations/20260508000001_baseline.up.sql"),
        include_str!("../../../../../migrations/20260508000002_add_rbac_timestamps.up.sql"),
    ]
}

fn test_database_name() -> String {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    let sequence = NEXT_TEST_DB_ID.fetch_add(1, Ordering::Relaxed);
    format!("hook_rbac_test_{}_{}_{}", std::process::id(), timestamp, sequence)
}
