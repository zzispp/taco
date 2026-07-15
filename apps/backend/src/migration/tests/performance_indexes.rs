use sqlx::{PgPool, query_scalar};

use super::{TestDatabase, rollback_from, up};

const PERFORMANCE_INDEX_MIGRATION: i64 = 20260715000003;

const INDEX_SPECS: &[IndexSpec] = &[
    IndexSpec::new("idx_sys_user_role_role_user", "sys_user_role", &["(role_id, user_id)"]),
    IndexSpec::new("idx_sys_user_post_post_user", "sys_user_post", &["(post_id, user_id)"]),
    IndexSpec::new("idx_sys_role_menu_menu_role", "sys_role_menu", &["(menu_id, role_id)"]),
    IndexSpec::new("idx_sys_role_dept_dept_role", "sys_role_dept", &["(dept_id, role_id)"]),
    IndexSpec::new(
        "idx_sys_user_active_status_create_time",
        "sys_user",
        &["(status, create_time, user_id)", "WHERE (del_flag = '0'::bpchar)"],
    ),
    IndexSpec::new("idx_sys_user_session_login_time", "sys_user_session", &["(login_time DESC, token_id)"]),
];

#[tokio::test]
async fn performance_indexes_match_read_paths_and_rollback() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();

    assert_index_contracts(database.pool()).await;
    rollback_from(database.pool(), PERFORMANCE_INDEX_MIGRATION).await;
    for spec in INDEX_SPECS {
        assert_eq!(index_definition(database.pool(), spec).await, None, "{} survived rollback", spec.name);
    }

    database.drop().await;
}

async fn assert_index_contracts(pool: &PgPool) {
    for spec in INDEX_SPECS {
        let definition = index_definition(pool, spec).await.unwrap_or_else(|| panic!("missing index {}", spec.name));
        for fragment in spec.fragments {
            assert!(definition.contains(fragment), "index {} does not contain {fragment}: {definition}", spec.name);
        }
    }
}

async fn index_definition(pool: &PgPool, spec: &IndexSpec) -> Option<String> {
    query_scalar("SELECT indexdef FROM pg_indexes WHERE schemaname='public' AND tablename=$1 AND indexname=$2")
        .bind(spec.table)
        .bind(spec.name)
        .fetch_optional(pool)
        .await
        .unwrap()
}

struct IndexSpec {
    name: &'static str,
    table: &'static str,
    fragments: &'static [&'static str],
}

impl IndexSpec {
    const fn new(name: &'static str, table: &'static str, fragments: &'static [&'static str]) -> Self {
        Self { name, table, fragments }
    }
}
