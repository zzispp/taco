use std::{
    ffi::OsString,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use configuration::Settings;
use kernel::pagination::CursorPageRequest;
use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions, query, query_scalar};
use url::Url;

use crate::application::{FileAccessScope, FileManagementRepository, FileScopeMode, FileSpaceQuery, UpdateSpaceCommand};
use crate::domain::{ByteSize, SpaceId};

use super::StorageFileRepository;

static NEXT_DATABASE_ID: AtomicU64 = AtomicU64::new(0);

#[path = "repository_tests/batch.rs"]
mod batch;
#[path = "repository_tests/directory_trail.rs"]
mod directory_trail;
#[path = "repository_tests/overview.rs"]
mod overview;
#[path = "repository_tests/upload.rs"]
mod upload;
#[path = "repository_tests/upload_idempotency.rs"]
mod upload_idempotency;
#[path = "repository_tests/upload_lifecycle.rs"]
mod upload_lifecycle;

#[tokio::test]
async fn list_spaces_projects_virtual_users_and_quota_update_materializes_one_space() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_department(database.pool(), "dept-1", "0").await;
    insert_department(database.pool(), "dept-10", "0").await;
    insert_user(database.pool(), "actor", Some("dept-1"), "Actor").await;
    insert_user(database.pool(), "peer", Some("dept-1"), "Peer").await;
    insert_user(database.pool(), "outside", Some("dept-10"), "Outside").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let scope = FileAccessScope::scoped("actor", FileScopeMode::Department, Some("dept-1".into()), Vec::new());

    let page = repository
        .list_spaces(&scope, FileSpaceQuery::default(), page(), ByteSize::from_bytes(20))
        .await
        .unwrap();
    let mut owner_ids = page.items.iter().map(|item| item.owner_user_id.as_str()).collect::<Vec<_>>();
    owner_ids.sort_unstable();
    assert_eq!(owner_ids, vec!["actor", "peer"]);
    for item in &page.items {
        assert_eq!(
            (item.logical_asset_size, item.managed_physical_usage, item.reserved_bytes, item.quota_bytes),
            (0, 0, 0, 20)
        );
    }
    assert_eq!(
        query_scalar::<_, i64>("SELECT COUNT(*) FROM file_space")
            .fetch_one(database.pool())
            .await
            .unwrap(),
        0
    );

    let updated = repository
        .update_space(
            &scope,
            SpaceId::new("peer").unwrap(),
            UpdateSpaceCommand { quota_bytes: Some(30) },
            ByteSize::from_bytes(20),
        )
        .await
        .unwrap();
    assert_eq!((updated.owner_user_id.as_str(), updated.quota_bytes), ("peer", 30));
    assert_eq!(
        query_scalar::<_, i64>("SELECT COUNT(*) FROM file_space")
            .fetch_one(database.pool())
            .await
            .unwrap(),
        1
    );
    database.drop().await;
}

#[tokio::test]
async fn virtual_space_scope_honors_self_custom_and_department_tree_boundaries() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    for (id, ancestors) in [("1", "0"), ("10", "0"), ("child", "0,1"), ("other", "0,10")] {
        insert_department(database.pool(), id, ancestors).await;
    }
    for (id, department) in [("actor", "10"), ("parent", "1"), ("child-user", "child"), ("other-user", "other")] {
        insert_user(database.pool(), id, Some(department), id).await;
    }
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    assert_eq!(owners(&repository, FileAccessScope::self_only("actor", Some("10".into()))).await, vec!["actor"]);
    assert_eq!(
        owners(
            &repository,
            FileAccessScope::scoped("actor", FileScopeMode::Custom, Some("10".into()), vec!["child".into()])
        )
        .await,
        vec!["actor", "child-user"]
    );
    assert_eq!(
        owners(
            &repository,
            FileAccessScope::scoped("actor", FileScopeMode::DepartmentAndChildren, Some("1".into()), Vec::new())
        )
        .await,
        vec!["actor", "child-user", "parent"]
    );
    assert_eq!(
        owners(&repository, FileAccessScope::all("actor")).await,
        vec!["actor", "child-user", "other-user", "parent"]
    );
    database.drop().await;
}

async fn owners(repository: &StorageFileRepository, scope: FileAccessScope) -> Vec<String> {
    let mut owners = repository
        .list_spaces(&scope, FileSpaceQuery::default(), page(), ByteSize::from_bytes(20))
        .await
        .unwrap()
        .items
        .into_iter()
        .map(|item| item.owner_user_id)
        .collect::<Vec<_>>();
    owners.sort();
    owners
}

const fn page() -> CursorPageRequest {
    CursorPageRequest { limit: 100, cursor: None }
}

async fn insert_department(pool: &PgPool, id: &str, ancestors: &str) {
    query("INSERT INTO sys_dept(dept_id,parent_id,ancestors,dept_name,create_time) VALUES($1,'0',$2,$1,CURRENT_TIMESTAMP)")
        .bind(id)
        .bind(ancestors)
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_user(pool: &PgPool, id: &str, department: Option<&str>, name: &str) {
    query("INSERT INTO sys_user(user_id,dept_id,user_name,nick_name,password,create_time) VALUES($1,$2,$1,$3,'hash',CURRENT_TIMESTAMP)")
        .bind(id)
        .bind(department)
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
}

struct TestDatabase {
    admin: PgPool,
    pool: PgPool,
    name: String,
}

impl TestDatabase {
    async fn create() -> Self {
        let configured = load_local_settings()
            .database_url()
            .unwrap_or_else(|error| panic!("test configuration database connection is invalid: {error}"));
        let url = Url::parse(&configured).unwrap_or_else(|error| panic!("test configuration database connection is not a valid URL: {error}"));
        assert!(
            matches!(url.scheme(), "postgres" | "postgresql"),
            "test configuration database connection must use PostgreSQL"
        );
        let mut admin_url = url.clone();
        admin_url.set_path("postgres");
        admin_url.set_fragment(None);
        let admin = PgPoolOptions::new().max_connections(1).connect(admin_url.as_str()).await.unwrap();
        let name = database_name();
        query(AssertSqlSafe(format!(r#"CREATE DATABASE "{name}""#))).execute(&admin).await.unwrap();
        let mut database_url = url;
        database_url.set_path(&name);
        database_url.set_fragment(None);
        let pool = PgPoolOptions::new().max_connections(5).connect(database_url.as_str()).await.unwrap();
        Self { admin, pool, name }
    }

    const fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn drop(self) {
        self.pool.close().await;
        query("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname=$1 AND pid<>pg_backend_pid()")
            .bind(&self.name)
            .execute(&self.admin)
            .await
            .unwrap();
        query(AssertSqlSafe(format!(r#"DROP DATABASE IF EXISTS "{}""#, self.name)))
            .execute(&self.admin)
            .await
            .unwrap();
        self.admin.close().await;
    }
}

fn load_local_settings() -> Settings {
    let path = local_configuration_path();
    Settings::load_from_args(vec![OsString::from("taco"), OsString::from("--config"), path.clone().into_os_string()])
        .unwrap_or_else(|error| panic!("failed to load local test configuration {}: {error}", path.display()))
}

fn local_configuration_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/config.yaml")
}

async fn migrate(pool: &PgPool) {
    let migrations = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
    sqlx::migrate::Migrator::new(migrations.as_path()).await.unwrap().run(pool).await.unwrap();
}

fn database_name() -> String {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    let id = NEXT_DATABASE_ID.fetch_add(1, Ordering::Relaxed);
    format!("taco_file_test_{}_{}_{}", std::process::id(), time, id)
}
