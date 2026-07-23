use sqlx::{PgPool, query_as, query_scalar};

const SYSTEM_ADMINISTRATOR_ROLE_ID: &str = "admin";

pub(super) async fn assert_system_administrator_role_seed(pool: &PgPool) {
    assert_system_administrator_role(pool).await;
    assert_system_administrator_menu_bindings(pool).await;
}

async fn assert_system_administrator_role(pool: &PgPool) {
    let roles: Vec<(String, String, String, bool)> = query_as("SELECT role_name,role_key,data_scope,system FROM sys_role WHERE del_flag='0' ORDER BY role_id")
        .fetch_all(pool)
        .await
        .unwrap();

    assert_eq!(roles, vec![("系统管理员".into(), SYSTEM_ADMINISTRATOR_ROLE_ID.into(), "1".into(), true)]);
}

async fn assert_system_administrator_menu_bindings(pool: &PgPool) {
    let menu_binding_count: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id=$1")
        .bind(SYSTEM_ADMINISTRATOR_ROLE_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    let menu_count: i64 = query_scalar("SELECT COUNT(*) FROM sys_menu").fetch_one(pool).await.unwrap();

    assert_eq!(menu_binding_count, menu_count);
    assert_eq!(
        query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_role_menu").fetch_one(pool).await.unwrap(),
        menu_count
    );
    assert_eq!(query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_role_dept").fetch_one(pool).await.unwrap(), 0);
    assert_eq!(query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_user_post").fetch_one(pool).await.unwrap(), 0);
}
