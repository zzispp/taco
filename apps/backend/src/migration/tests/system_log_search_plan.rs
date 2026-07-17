use serde_json::Value;
use sqlx::{AssertSqlSafe, PgPool, query, query_scalar};

use super::{TestDatabase, up};

const FIXTURE_ROWS: i32 = 50_000;
const NGRAM_INDEX: &str = "search_ngrams_idx";
const TARGET_INDEX: &str = "target_occurred_at_id_idx";

#[tokio::test]
async fn system_log_short_cjk_and_target_prefix_queries_use_their_indexes() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    seed_search_fixture(database.pool()).await;

    let ngram_plan = explain(
        database.pool(),
        "SELECT id FROM sys_system_log WHERE search_ngrams @> system_log_search_ngrams('中文')",
    )
    .await;
    let target_plan = explain(
        database.pool(),
        "SELECT id FROM sys_system_log WHERE target='user' OR target LIKE 'user::%' ESCAPE '\\'",
    )
    .await;

    assert_plan_uses_index(&ngram_plan, NGRAM_INDEX);
    assert_plan_uses_index(&target_plan, TARGET_INDEX);
    database.drop().await;
}

async fn seed_search_fixture(pool: &PgPool) {
    query("SELECT ensure_system_log_partition(TIMESTAMPTZ '2026-07-16 12:00:00+00')")
        .execute(pool)
        .await
        .unwrap();
    query(
        "INSERT INTO sys_system_log (id,occurred_at,level,target,message,fields) SELECT 'search-' || value,TIMESTAMPTZ '2026-07-16 12:00:00+00' + value * INTERVAL '1 microsecond','info',CASE WHEN value <= 2 THEN 'user::api::handlers' ELSE 'audit::api' END,CASE WHEN value = 1 THEN '中文请求完成' ELSE 'unrelated message' END,'{}'::jsonb FROM generate_series(1,$1) AS value",
    )
    .bind(FIXTURE_ROWS)
    .execute(pool)
    .await
    .unwrap();
    query("ANALYZE sys_system_log").execute(pool).await.unwrap();
}

async fn explain(pool: &PgPool, statement: &str) -> Value {
    query_scalar::<_, Value>(AssertSqlSafe(format!("EXPLAIN (COSTS OFF, FORMAT JSON) {statement}")))
        .fetch_one(pool)
        .await
        .unwrap()
}

fn assert_plan_uses_index(plan: &Value, index: &str) {
    assert!(plan.to_string().contains(index), "expected {index} in query plan: {plan}");
}
