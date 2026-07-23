use sqlx::{PgPool, query, query_as, query_scalar};

use super::{OBJECT_ID, OWNER_ID, SPACE_ID, insert_entry};

pub(super) async fn assert_avatar_reference_invariants(pool: &PgPool) {
    query("UPDATE file_object SET content_type='image/png' WHERE object_id=$1")
        .bind(OBJECT_ID)
        .execute(pool)
        .await
        .unwrap();
    insert_entry(pool, "entry-avatar", "file", "avatar.png", "avatar.png", Some(OBJECT_ID), "active")
        .await
        .unwrap();
    query("UPDATE sys_user SET avatar_file_id='entry-avatar',avatar_version=1 WHERE user_id=$1")
        .bind(OWNER_ID)
        .execute(pool)
        .await
        .unwrap();

    assert_current_avatar_cannot_be_invalidated(pool).await;
    assert_replacement_atomically_retires_previous_avatar(pool).await;
}

async fn assert_current_avatar_cannot_be_invalidated(pool: &PgPool) {
    let select_trashed = query("UPDATE sys_user SET avatar_file_id='entry-trashed' WHERE user_id=$1")
        .bind(OWNER_ID)
        .execute(pool)
        .await;
    let trash_avatar = query("UPDATE file_entry SET status='trashed',trashed_at=CURRENT_TIMESTAMP WHERE entry_id='entry-avatar'")
        .execute(pool)
        .await;
    let delete_avatar = query("DELETE FROM file_entry WHERE entry_id='entry-avatar'").execute(pool).await;
    let legacy_column_count: i64 =
        query_scalar("SELECT COUNT(*) FROM information_schema.columns WHERE table_schema='public' AND table_name='sys_user' AND column_name='avatar'")
            .fetch_one(pool)
            .await
            .unwrap();

    assert!(select_trashed.is_err());
    assert!(trash_avatar.is_err());
    assert!(delete_avatar.is_err());
    assert_eq!(legacy_column_count, 0);
}

async fn assert_replacement_atomically_retires_previous_avatar(pool: &PgPool) {
    insert_entry(
        pool,
        "entry-avatar-next",
        "file",
        "avatar-next.png",
        "avatar-next.png",
        Some(OBJECT_ID),
        "active",
    )
    .await
    .unwrap();
    assert_business_reference_blocks_replacement(pool).await;
    let replace_avatar = query("UPDATE sys_user SET avatar_file_id='entry-avatar-next',avatar_version=avatar_version+1 WHERE user_id=$1")
        .bind(OWNER_ID)
        .execute(pool)
        .await;
    let old_status: String = query_scalar("SELECT status FROM file_entry WHERE entry_id='entry-avatar'")
        .fetch_one(pool)
        .await
        .unwrap();
    let usage: (i64, i64) = query_as("SELECT active_bytes,trashed_bytes FROM file_space WHERE space_id=$1")
        .bind(SPACE_ID)
        .fetch_one(pool)
        .await
        .unwrap();

    assert!(replace_avatar.is_ok());
    assert_eq!(old_status, "trashed");
    assert_eq!(usage, (4, 8));
}

async fn assert_business_reference_blocks_replacement(pool: &PgPool) {
    query("INSERT INTO file_business_reference (reference_id,entry_id,context_key,reference_key,created_at) VALUES ('avatar-reference','entry-avatar','test','avatar-reference',CURRENT_TIMESTAMP)")
        .execute(pool)
        .await
        .unwrap();
    let replacement = query("UPDATE sys_user SET avatar_file_id='entry-avatar-next',avatar_version=avatar_version+1 WHERE user_id=$1")
        .bind(OWNER_ID)
        .execute(pool)
        .await;
    let current: (String, String) =
        query_as("SELECT avatar_file_id,(SELECT status FROM file_entry WHERE entry_id='entry-avatar') FROM sys_user WHERE user_id=$1")
            .bind(OWNER_ID)
            .fetch_one(pool)
            .await
            .unwrap();
    query("DELETE FROM file_business_reference WHERE reference_id='avatar-reference'")
        .execute(pool)
        .await
        .unwrap();

    assert!(replacement.is_err());
    assert_eq!(current, ("entry-avatar".into(), "active".into()));
}
