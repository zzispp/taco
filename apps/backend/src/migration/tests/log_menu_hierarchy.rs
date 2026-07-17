use sqlx::{PgPool, query, query_scalar};

use super::{TestDatabase, down, up};

const MIGRATIONS_THROUGH_LOG_MENU_HIERARCHY: u32 = 25;
const CHILD_ONLY_ROLE_ID: &str = "log-child-only";
const EXISTING_PARENT_ROLE_ID: &str = "log-parent-existing";

#[tokio::test]
async fn log_menu_hierarchy_grants_new_parent_to_roles_with_scheduler_logs() {
    let database = TestDatabase::create().await;
    up(database.pool(), Some(MIGRATIONS_THROUGH_LOG_MENU_HIERARCHY)).await.unwrap();
    insert_role_bindings(database.pool()).await;

    up(database.pool(), Some(2)).await.unwrap();

    assert_parent_binding(database.pool(), CHILD_ONLY_ROLE_ID, 1).await;
    assert_parent_binding(database.pool(), EXISTING_PARENT_ROLE_ID, 1).await;
    down(database.pool(), Some(1)).await.unwrap();
    assert_parent_binding(database.pool(), CHILD_ONLY_ROLE_ID, 0).await;
    assert_parent_binding(database.pool(), EXISTING_PARENT_ROLE_ID, 1).await;

    database.drop().await;
}

async fn insert_role_bindings(pool: &PgPool) {
    for (role_id, include_parent) in [(CHILD_ONLY_ROLE_ID, false), (EXISTING_PARENT_ROLE_ID, true)] {
        query("INSERT INTO sys_role (role_id,role_name,role_key,role_sort,status,create_time) VALUES ($1,$1,$1,10,'0',CURRENT_TIMESTAMP)")
            .bind(role_id)
            .execute(pool)
            .await
            .unwrap();
        query("INSERT INTO sys_role_menu (role_id,menu_id) VALUES ($1,'109')")
            .bind(role_id)
            .execute(pool)
            .await
            .unwrap();
        if include_parent {
            query("INSERT INTO sys_role_menu (role_id,menu_id) VALUES ($1,'111')")
                .bind(role_id)
                .execute(pool)
                .await
                .unwrap();
        }
    }
}

async fn assert_parent_binding(pool: &PgPool, role_id: &str, expected: i64) {
    let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id=$1 AND menu_id='111'")
        .bind(role_id)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, expected, "unexpected log management binding for role {role_id}");
}
