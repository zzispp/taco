use sqlx::{PgPool, query_as};

use super::{TestDatabase, migrate_through, rollback_from, up};

const FUNCTION_SECURITY_MIGRATION_VERSION: i64 = 20260805000001;
const PARTITION_FUNCTIONS: &[&str] = &[
    "public.ensure_system_log_partition(timestamp with time zone)",
    "public.drop_expired_system_log_partition(text,timestamp with time zone)",
];
const SECURE_SEARCH_PATH: &str = "search_path=pg_catalog, public";

type FunctionSecurity = (bool, Option<Vec<String>>, bool);

#[tokio::test]
async fn partition_maintenance_functions_use_restricted_definer_security() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();

    for signature in PARTITION_FUNCTIONS {
        let security = function_security(database.pool(), signature).await;
        assert!(security.0, "{signature} must use SECURITY DEFINER");
        assert_eq!(security.1, Some(vec![SECURE_SEARCH_PATH.into()]), "unexpected search_path for {signature}");
        assert!(!security.2, "PUBLIC must not execute {signature}");
    }

    database.drop().await;
}

#[tokio::test]
async fn partition_maintenance_function_security_rolls_back_explicitly() {
    let database = TestDatabase::create().await;
    migrate_through(database.pool(), FUNCTION_SECURITY_MIGRATION_VERSION).await;

    rollback_from(database.pool(), FUNCTION_SECURITY_MIGRATION_VERSION).await;

    for signature in PARTITION_FUNCTIONS {
        let security = function_security(database.pool(), signature).await;
        assert!(!security.0, "{signature} must return to SECURITY INVOKER");
        assert_eq!(security.1, None, "{signature} must reset search_path");
        assert!(security.2, "PUBLIC execute must be restored for {signature}");
    }

    database.drop().await;
}

async fn function_security(pool: &PgPool, signature: &str) -> FunctionSecurity {
    query_as(
        "SELECT procedure.prosecdef, procedure.proconfig, \
                EXISTS(SELECT 1 FROM aclexplode(COALESCE(procedure.proacl, acldefault('f', procedure.proowner))) privilege \
                       WHERE privilege.grantee=0 AND privilege.privilege_type='EXECUTE') \
         FROM pg_proc procedure WHERE procedure.oid=to_regprocedure($1)",
    )
    .bind(signature)
    .fetch_one(pool)
    .await
    .unwrap()
}
