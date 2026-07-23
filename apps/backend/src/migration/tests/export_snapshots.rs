use kernel::pagination::CursorPageRequest;
use rbac::{
    application::{RbacRepository, RoleExportRequest, RoleExportSink, RoleListFilter},
    domain::{DataScope, DataScopeFilter, Role},
    infra::StorageRbacRepository,
};
use sqlx::{PgPool, query};
use storage::Database;
use system::{
    application::{PostListFilter, SystemExportBatch, SystemExportKind, SystemExportRequest, SystemExportSink, SystemRepository},
    infra::StorageSystemRepository,
};
use user::{
    application::{UserExportRequest, UserExportSink, UserListFilter, UserRepository},
    domain::{User, UserId},
    infra::StorageUserRepository,
};

use super::{TestDatabase, up};

const USER_INSERT: &str = "INSERT INTO sys_user (user_id,user_name,nick_name,email,password,create_time) VALUES ('snapshot-user-c','snapshot-user-c','Snapshot C','snapshot-user-c@example.test','hash',TIMESTAMPTZ '2026-01-03 00:00:00+00')";
const ROLE_INSERT: &str = "INSERT INTO sys_role (role_id,role_name,role_key,role_sort,status,create_time) VALUES ('snapshot-role-c','snapshot export role c','snapshot-role-c',3,'0',CURRENT_TIMESTAMP)";
const POST_INSERT: &str = "INSERT INTO sys_post (post_id,post_code,post_name,post_sort,status,create_time) VALUES ('snapshot-post-c','snapshot-post-c','Snapshot C',3,'0',CURRENT_TIMESTAMP)";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn repository_exports_stream_one_consistent_snapshot() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();

    assert_user_export(database.pool()).await;
    assert_profile_groups(database.pool()).await;
    assert_role_export(database.pool()).await;
    assert_system_export(database.pool()).await;

    database.drop().await;
}

async fn assert_profile_groups(pool: &PgPool) {
    execute_sql(
        pool,
        "INSERT INTO sys_dept (dept_id,dept_name,create_time) VALUES ('snapshot-dept','Snapshot Department',CURRENT_TIMESTAMP)",
    )
    .await;
    execute_sql(pool, "INSERT INTO sys_role (role_id,role_name,role_key,role_sort,status,create_time) VALUES ('snapshot-profile-role','Snapshot Role','snapshot-profile-role',1,'0',CURRENT_TIMESTAMP)").await;
    execute_sql(pool, "INSERT INTO sys_post (post_id,post_code,post_name,post_sort,status,create_time) VALUES ('snapshot-profile-post','snapshot-profile-post','Snapshot Post',1,'0',CURRENT_TIMESTAMP)").await;
    execute_sql(pool, "UPDATE sys_user SET dept_id='snapshot-dept' WHERE user_id='snapshot-user-a'").await;
    execute_sql(pool, "INSERT INTO sys_user_role VALUES ('snapshot-user-a','snapshot-profile-role')").await;
    execute_sql(pool, "INSERT INTO sys_user_post VALUES ('snapshot-user-a','snapshot-profile-post')").await;

    let repository = StorageUserRepository::new(Database::new(pool.clone()));
    let groups = repository.profile_groups(UserId("snapshot-user-a".into())).await.unwrap();

    assert_eq!(groups.role_group, "Snapshot Role");
    assert_eq!(groups.post_group, "Snapshot Post");
    assert_eq!(groups.dept_name.as_deref(), Some("Snapshot Department"));
}

async fn execute_sql(pool: &PgPool, statement: &'static str) {
    query(statement).execute(pool).await.unwrap();
}

async fn assert_user_export(pool: &PgPool) {
    query("INSERT INTO sys_user (user_id,user_name,nick_name,email,password,create_time) VALUES ('snapshot-user-a','snapshot-user-a','Snapshot A','snapshot-user-a@example.test','hash',TIMESTAMPTZ '2026-01-01 00:00:00+00'),('snapshot-user-b','snapshot-user-b','Snapshot B','snapshot-user-b@example.test','hash',TIMESTAMPTZ '2026-01-02 00:00:00+00')")
        .execute(pool)
        .await
        .unwrap();
    let repository = StorageUserRepository::new(Database::new(pool.clone()));
    let mut sink = SnapshotProbe::new(pool.clone(), USER_INSERT);

    repository
        .export_users(
            UserExportRequest {
                filter: user_filter(),
                scope: all_data_scope(),
                batch_size: 1,
            },
            &mut sink,
        )
        .await
        .unwrap();

    assert_eq!(sink.batches, vec![vec!["snapshot-user-a"], vec!["snapshot-user-b"]]);
}

async fn assert_role_export(pool: &PgPool) {
    query("INSERT INTO sys_role (role_id,role_name,role_key,role_sort,status,create_time) VALUES ('snapshot-role-a','snapshot export role a','snapshot-role-a',1,'0',CURRENT_TIMESTAMP),('snapshot-role-b','snapshot export role b','snapshot-role-b',2,'0',CURRENT_TIMESTAMP)")
        .execute(pool)
        .await
        .unwrap();
    let repository = StorageRbacRepository::new(Database::new(pool.clone()));
    let mut sink = SnapshotProbe::new(pool.clone(), ROLE_INSERT);

    repository
        .export_roles(
            RoleExportRequest {
                filter: role_filter(),
                scope: None,
                batch_size: 1,
            },
            &mut sink,
        )
        .await
        .unwrap();

    assert_eq!(sink.batches, vec![vec!["snapshot-role-a"], vec!["snapshot-role-b"]]);
}

async fn assert_system_export(pool: &PgPool) {
    query("INSERT INTO sys_post (post_id,post_code,post_name,post_sort,status,create_time) VALUES ('snapshot-post-a','snapshot-post-a','Snapshot A',1,'0',CURRENT_TIMESTAMP),('snapshot-post-b','snapshot-post-b','Snapshot B',2,'0',CURRENT_TIMESTAMP)")
        .execute(pool)
        .await
        .unwrap();
    let repository = StorageSystemRepository::new(Database::new(pool.clone()));
    let mut sink = SnapshotProbe::new(pool.clone(), POST_INSERT);

    repository
        .export(
            SystemExportRequest {
                kind: SystemExportKind::Posts(post_filter()),
                batch_size: 1,
            },
            &mut sink,
        )
        .await
        .unwrap();

    assert_eq!(sink.batches, vec![vec!["snapshot-post-a"], vec!["snapshot-post-b"]]);
}

struct SnapshotProbe {
    pool: PgPool,
    insert_sql: &'static str,
    batches: Vec<Vec<String>>,
    mutated: bool,
}

impl SnapshotProbe {
    fn new(pool: PgPool, insert_sql: &'static str) -> Self {
        Self {
            pool,
            insert_sql,
            batches: Vec::new(),
            mutated: false,
        }
    }

    fn record(&mut self, values: Vec<String>) {
        self.batches.push(values);
        if self.mutated {
            return;
        }
        self.mutated = true;
        let pool = self.pool.clone();
        let sql = self.insert_sql;
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(async move { query(sql).execute(&pool).await.unwrap() }));
    }
}

impl UserExportSink for SnapshotProbe {
    fn append(&mut self, users: &[User]) -> user::application::AppResult<()> {
        self.record(users.iter().map(|user| user.username.clone()).collect());
        Ok(())
    }
}

impl RoleExportSink for SnapshotProbe {
    fn append(&mut self, roles: &[Role]) -> rbac::application::RbacResult<()> {
        self.record(roles.iter().map(|role| role.role_id.clone()).collect());
        Ok(())
    }
}

impl SystemExportSink for SnapshotProbe {
    fn append(&mut self, batch: SystemExportBatch) -> system::application::SystemResult<()> {
        let SystemExportBatch::Posts(posts) = batch else {
            panic!("system post export emitted a different batch kind");
        };
        self.record(posts.iter().map(|post| post.post_id.clone()).collect());
        Ok(())
    }
}

fn user_filter() -> UserListFilter {
    UserListFilter {
        page: export_cursor_request(),
        username: Some("snapshot-user".into()),
        nick_name: None,
        phonenumber: None,
        email: None,
        sex: None,
        status: None,
        dept_id: None,
        dept_name: None,
        post_ids: Vec::new(),
        role_ids: Vec::new(),
        begin_time: None,
        end_time: None,
    }
}

fn all_data_scope() -> DataScopeFilter {
    DataScopeFilter {
        data_scope: DataScope::All,
        user_id: "snapshot-exporter".into(),
        dept_id: None,
        dept_ids: Vec::new(),
    }
}

fn role_filter() -> RoleListFilter {
    RoleListFilter {
        page: export_cursor_request(),
        role_name: Some("snapshot export role".into()),
        role_key: None,
        status: None,
        system: None,
        begin_time: None,
        end_time: None,
    }
}

fn post_filter() -> PostListFilter {
    PostListFilter {
        page: export_cursor_request(),
        post_code: Some("snapshot-post".into()),
        post_name: None,
        status: None,
        remark: None,
        begin_time: None,
        end_time: None,
    }
}

const fn export_cursor_request() -> CursorPageRequest {
    CursorPageRequest { limit: 1, cursor: None }
}
