use rbac::{application::RbacRepository, domain::RoleDataScopeInput, infra::StorageRbacRepository};
use sqlx::{PgPool, query, query_scalar};
use storage::Database;

use user::{application::UserRepository, domain::UserId, infra::StorageUserRepository};

use self::fixtures::{MenuFixture, RelationFixture, RoleFixture, UserFixture};
use super::{TestDatabase, bootstrap_system_administrator, up};

#[path = "data_integrity/fixtures.rs"]
mod fixtures;

const REUSABLE_USER_ID: &str = "integrity-u2";
const CLEANUP_ROLE_ID: &str = "integrity-r-clean";
const CLEANUP_MENU_ID: &str = "integrity-m-clean";
const CLEANUP_POST_ID: &str = "integrity-p";
const CLEANUP_DEPT_ID: &str = "integrity-d";
const SCOPE_ROLE_ID: &str = "integrity-r-scope";

#[tokio::test]
async fn integrity_migration_enforces_identity_and_relation_semantics() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    bootstrap_system_administrator(database.pool()).await;

    assert_user_identity_semantics(database.pool()).await;
    assert_role_uniqueness(database.pool()).await;
    assert_menu_uniqueness(database.pool()).await;
    assert_soft_delete_relation_cleanup(database.pool()).await;

    database.drop().await;
}

async fn assert_user_identity_semantics(pool: &PgPool) {
    UserFixture::active("integrity-u1", "reusable", "Mixed@Example.COM")
        .with_phone("13900000001")
        .insert(pool)
        .await;
    let email_conflict = UserFixture::active("integrity-u-email", "other", "mixed@example.com").insert_result(pool).await;
    assert!(email_conflict.is_err());

    query("UPDATE sys_user SET del_flag='2' WHERE user_id='integrity-u1'")
        .execute(pool)
        .await
        .unwrap();
    UserFixture::active(REUSABLE_USER_ID, "reusable", "MIXED@example.com")
        .with_phone("13900000001")
        .insert(pool)
        .await;

    let stored_email = query_scalar::<_, String>("SELECT email FROM sys_user WHERE user_id=$1")
        .bind(REUSABLE_USER_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(stored_email, "mixed@example.com");
}

async fn assert_role_uniqueness(pool: &PgPool) {
    RoleFixture::active("integrity-r-deleted", "Reserved role", "reserved-role")
        .with_del_flag("2")
        .insert(pool)
        .await;
    assert!(
        RoleFixture::active("integrity-r-name", "Reserved role", "other-role")
            .insert_result(pool)
            .await
            .is_err()
    );
    assert!(
        RoleFixture::active("integrity-r-key", "Other role", "reserved-role")
            .insert_result(pool)
            .await
            .is_err()
    );
}

async fn assert_menu_uniqueness(pool: &PgPool) {
    MenuFixture {
        id: "integrity-m1",
        name: "Integrity menu",
        path: "/integrity",
        route_name: "IntegrityRoute",
    }
    .insert(pool)
    .await;
    assert!(
        MenuFixture {
            id: "integrity-m-name",
            name: "Integrity menu",
            path: "/other",
            route_name: "OtherRoute",
        }
        .insert_result(pool)
        .await
        .is_err()
    );
    assert!(
        MenuFixture {
            id: "integrity-m-path",
            name: "Other menu",
            path: "/integrity",
            route_name: "OtherRoute2",
        }
        .insert_result(pool)
        .await
        .is_err()
    );
    assert!(
        MenuFixture {
            id: "integrity-m-route",
            name: "Other menu 2",
            path: "/other-2",
            route_name: "IntegrityRoute",
        }
        .insert_result(pool)
        .await
        .is_err()
    );
}

async fn assert_soft_delete_relation_cleanup(pool: &PgPool) {
    prepare_cleanup_relations(pool).await;
    assert_user_relation_cleanup(pool).await;
    assert_role_relation_cleanup(pool).await;
    assert_non_custom_scope_cleanup(pool).await;
}

async fn prepare_cleanup_relations(pool: &PgPool) {
    RoleFixture::active(CLEANUP_ROLE_ID, "Cleanup role", "cleanup-role").insert(pool).await;
    MenuFixture {
        id: CLEANUP_MENU_ID,
        name: "Cleanup menu",
        path: "/cleanup",
        route_name: "CleanupRoute",
    }
    .insert(pool)
    .await;
    query("INSERT INTO sys_post (post_id,post_code,post_name,post_sort,status,create_time) VALUES ($1,$1,'Integrity',1,'0',CURRENT_TIMESTAMP)")
        .bind(CLEANUP_POST_ID)
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO sys_dept (dept_id,parent_id,ancestors,dept_name,create_time) VALUES ($1,'0','0','Integrity',CURRENT_TIMESTAMP)")
        .bind(CLEANUP_DEPT_ID)
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO sys_user_role VALUES ($1,$2)")
        .bind(REUSABLE_USER_ID)
        .bind(CLEANUP_ROLE_ID)
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO sys_user_post VALUES ($1,$2)")
        .bind(REUSABLE_USER_ID)
        .bind(CLEANUP_POST_ID)
        .execute(pool)
        .await
        .unwrap();
}

async fn assert_user_relation_cleanup(pool: &PgPool) {
    StorageUserRepository::new(Database::new(pool.clone()))
        .delete(UserId(REUSABLE_USER_ID.into()))
        .await
        .unwrap();
    assert_eq!(RelationFixture::new("sys_user_role", "user_id", REUSABLE_USER_ID).count(pool).await, 0);
    assert_eq!(RelationFixture::new("sys_user_post", "user_id", REUSABLE_USER_ID).count(pool).await, 0);
}

async fn assert_role_relation_cleanup(pool: &PgPool) {
    query("INSERT INTO sys_user_role VALUES ($1,$2)")
        .bind(REUSABLE_USER_ID)
        .bind(CLEANUP_ROLE_ID)
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO sys_role_menu VALUES ($1,$2)")
        .bind(CLEANUP_ROLE_ID)
        .bind(CLEANUP_MENU_ID)
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO sys_role_dept VALUES ($1,$2)")
        .bind(CLEANUP_ROLE_ID)
        .bind(CLEANUP_DEPT_ID)
        .execute(pool)
        .await
        .unwrap();
    let repository = StorageRbacRepository::new(Database::new(pool.clone()));
    assert!(!repository.role_has_users(CLEANUP_ROLE_ID).await.unwrap());
    repository.delete_role(CLEANUP_ROLE_ID).await.unwrap();
    for table in ["sys_user_role", "sys_role_menu", "sys_role_dept"] {
        assert_eq!(RelationFixture::new(table, "role_id", CLEANUP_ROLE_ID).count(pool).await, 0);
    }
}

async fn assert_non_custom_scope_cleanup(pool: &PgPool) {
    RoleFixture::active(SCOPE_ROLE_ID, "Scope role", "scope-role").insert(pool).await;
    query("INSERT INTO sys_role_dept VALUES ($1,$2)")
        .bind(SCOPE_ROLE_ID)
        .bind(CLEANUP_DEPT_ID)
        .execute(pool)
        .await
        .unwrap();
    StorageRbacRepository::new(Database::new(pool.clone()))
        .update_role_data_scope(
            SCOPE_ROLE_ID,
            RoleDataScopeInput {
                data_scope: "3".into(),
                dept_check_strictly: false,
                dept_ids: vec![CLEANUP_DEPT_ID.into()],
            },
        )
        .await
        .unwrap();
    assert_eq!(RelationFixture::new("sys_role_dept", "role_id", SCOPE_ROLE_ID).count(pool).await, 0);
}
