use sqlx::postgres::PgPoolOptions;
use storage::Database;

#[test]
fn read_writes_are_idempotent_locked_and_batched() {
    assert!(super::MARK_READ_SQL.contains("ON CONFLICT ON CONSTRAINT uk_sys_notice_read_user_notice DO NOTHING"));
    assert!(super::MARK_ALL_READ_SQL.contains("UNNEST($1::text[],$2::text[])"));
    assert!(super::UNREAD_NOTICE_IDS_SQL.contains("status=$2"));
    assert!(super::UNREAD_NOTICE_IDS_SQL.contains("NOT EXISTS"));
    assert!(super::UNREAD_NOTICE_IDS_SQL.contains("FOR UPDATE OF n"));
}

#[tokio::test]
async fn batch_read_ids_are_unique_uuid_v7_values() {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://postgres:postgres@localhost/taco")
        .expect("lazy pool");
    let ids = super::next_read_ids(&Database::new(pool), 8);

    assert_eq!(ids.len(), 8);
    assert_eq!(ids.iter().collect::<std::collections::HashSet<_>>().len(), 8);
    assert!(ids.iter().all(|id| id.len() == 36 && id.as_bytes().get(14) == Some(&b'7')));
}
