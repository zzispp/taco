use sqlx::{PgPool, query};

use super::{TestDatabase, down, fresh, managed_table_exists, migrate_through};

mod avatar;

const OWNER_ID: &str = "file-owner";
const SPACE_ID: &str = "file-space";
const OBJECT_ID: &str = "file-object";
const FILE_MANAGEMENT_INTEGRATION_VERSION: i64 = 20260720000002;

#[tokio::test]
async fn file_management_schema_enforces_asset_and_upload_invariants() {
    let database = TestDatabase::create().await;
    let pool = database.pool();
    fresh(pool).await.unwrap();
    insert_owner_and_space(pool).await;

    assert_space_invariants(pool).await;
    assert_entry_invariants(pool).await;
    assert_metadata_invariants(pool).await;
    assert_upload_invariants(pool).await;
    assert_provider_cleanup_invariants(pool).await;
    avatar::assert_avatar_reference_invariants(pool).await;
    assert_soft_delete_archives_current_department(pool).await;

    database.drop().await;
}

#[tokio::test]
async fn file_management_migrations_roll_back_in_reverse_dependency_order() {
    let database = TestDatabase::create().await;
    let pool = database.pool();
    migrate_through(pool, FILE_MANAGEMENT_INTEGRATION_VERSION).await;

    down(pool, Some(2)).await.unwrap();

    let legacy_avatar_column: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM information_schema.columns WHERE table_schema='public' AND table_name='sys_user' AND column_name='avatar'")
            .fetch_one(pool)
            .await
            .unwrap();
    let file_jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sys_job WHERE task_key LIKE 'file.%'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(!managed_table_exists(pool, "file_entry").await);
    assert_eq!(legacy_avatar_column, 1);
    assert_eq!(file_jobs, 0);

    database.drop().await;
}

async fn assert_soft_delete_archives_current_department(pool: &PgPool) {
    query("INSERT INTO sys_dept(dept_id,parent_id,ancestors,dept_name,create_time) VALUES('archive-dept','0','0','Archive Dept',CURRENT_TIMESTAMP)")
        .execute(pool)
        .await
        .unwrap();
    query("UPDATE sys_user SET dept_id='archive-dept' WHERE user_id=$1")
        .bind(OWNER_ID)
        .execute(pool)
        .await
        .unwrap();
    query("UPDATE sys_user SET del_flag='2' WHERE user_id=$1")
        .bind(OWNER_ID)
        .execute(pool)
        .await
        .unwrap();

    let archived: (Option<String>, String, bool) = sqlx::query_as("SELECT owner_dept_id,status,archived_at IS NOT NULL FROM file_space WHERE space_id=$1")
        .bind(SPACE_ID)
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(archived, (Some("archive-dept".into()), "archived".into(), true));
}

async fn insert_owner_and_space(pool: &PgPool) {
    query(
        "INSERT INTO sys_user (user_id,user_name,nick_name,password,create_time) \
         VALUES ($1,'file-owner','File Owner','hash',CURRENT_TIMESTAMP)",
    )
    .bind(OWNER_ID)
    .execute(pool)
    .await
    .unwrap();
    query(
        "INSERT INTO file_space (space_id,owner_user_id,owner_dept_id,created_at,updated_at) \
         VALUES ($1,$2,'dept-snapshot',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .bind(SPACE_ID)
    .bind(OWNER_ID)
    .execute(pool)
    .await
    .unwrap();
}

async fn assert_space_invariants(pool: &PgPool) {
    let duplicate_owner = query(
        "INSERT INTO file_space (space_id,owner_user_id,created_at,updated_at) \
         VALUES ('duplicate-space',$1,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .bind(OWNER_ID)
    .execute(pool)
    .await;
    let negative_usage = query("UPDATE file_space SET reserved_bytes=-1 WHERE space_id=$1")
        .bind(SPACE_ID)
        .execute(pool)
        .await;

    assert!(duplicate_owner.is_err());
    assert!(negative_usage.is_err());
}

async fn assert_entry_invariants(pool: &PgPool) {
    insert_object(pool).await;
    insert_entry(pool, "entry-folder", "folder", "Documents", "documents", None, "active")
        .await
        .unwrap();

    let duplicate = insert_entry(pool, "entry-file", "file", "DOCUMENTS", "documents", Some(OBJECT_ID), "active").await;
    let trashed_duplicate = insert_entry(pool, "entry-trashed", "file", "DOCUMENTS", "documents", Some(OBJECT_ID), "trashed").await;
    let folder_with_object = insert_entry(
        pool,
        "entry-invalid-folder",
        "folder",
        "invalid-folder",
        "invalid-folder",
        Some(OBJECT_ID),
        "active",
    )
    .await;
    let file_without_object = insert_entry(pool, "entry-invalid-file", "file", "invalid-file", "invalid-file", None, "active").await;

    assert!(duplicate.is_err());
    assert!(trashed_duplicate.is_ok());
    assert!(folder_with_object.is_err());
    assert!(file_without_object.is_err());
}

async fn assert_metadata_invariants(pool: &PgPool) {
    query("INSERT INTO file_tag (tag_id,space_id,name,normalized_name,created_by,created_at) VALUES ('tag-1',$1,'Work','work',$2,CURRENT_TIMESTAMP)")
        .bind(SPACE_ID)
        .bind(OWNER_ID)
        .execute(pool)
        .await
        .unwrap();
    let duplicate_tag = query(
        "INSERT INTO file_tag (tag_id,space_id,name,normalized_name,created_by,created_at) \
         VALUES ('tag-2',$1,'WORK','work',$2,CURRENT_TIMESTAMP)",
    )
    .bind(SPACE_ID)
    .bind(OWNER_ID)
    .execute(pool)
    .await;
    assert!(duplicate_tag.is_err());
}

async fn assert_upload_invariants(pool: &PgPool) {
    insert_upload_session(pool, "upload-1", "intent-1", "open").await.unwrap();
    let duplicate_intent = insert_upload_session(pool, "upload-2", "intent-1", "open").await;
    let invalid_terminal = insert_upload_session(pool, "upload-3", "intent-3", "completed").await;
    query(
        "INSERT INTO file_upload_part (session_id,part_number,size_bytes,sha256,provider_part_ref,created_at) \
         VALUES ('upload-1',1,4,repeat('b',64),'part-1',CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await
    .unwrap();
    let invalid_part = query(
        "INSERT INTO file_upload_part (session_id,part_number,size_bytes,sha256,created_at) \
         VALUES ('upload-1',0,4,repeat('c',64),CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await;
    let completed_part_without_provider_ref = query(
        "INSERT INTO file_upload_part (session_id,part_number,size_bytes,sha256,created_at) \
         VALUES ('upload-1',2,4,repeat('c',64),CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await;

    assert!(duplicate_intent.is_err());
    assert!(invalid_terminal.is_err());
    assert!(invalid_part.is_err());
    assert!(completed_part_without_provider_ref.is_err());
}

async fn assert_provider_cleanup_invariants(pool: &PgPool) {
    let missing_object_key = query(
        "INSERT INTO file_provider_cleanup \
         (cleanup_id,provider_key,cleanup_kind,status,attempt_count,next_attempt_at,created_at,updated_at) \
         VALUES ('cleanup-invalid-object','local','object','pending',0,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await;
    let upload_with_object_key = query(
        "INSERT INTO file_provider_cleanup \
         (cleanup_id,provider_key,cleanup_kind,object_key,upload_ref,status,attempt_count,next_attempt_at,created_at,updated_at) \
         VALUES ('cleanup-invalid-upload','local','upload','unexpected','upload-ref','pending',0,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await;

    assert!(missing_object_key.is_err());
    assert!(upload_with_object_key.is_err());
}

async fn insert_object(pool: &PgPool) {
    query(
        "INSERT INTO file_object (object_id,provider_key,object_key,size_bytes,sha256,content_type,ref_count,status,created_at,updated_at) \
         VALUES ($1,'local','objects/test',4,repeat('a',64),'text/plain',2,'active',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .bind(OBJECT_ID)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_entry(
    pool: &PgPool,
    entry_id: &str,
    kind: &str,
    name: &str,
    normalized_name: &str,
    object_id: Option<&str>,
    status: &str,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    query(
        "INSERT INTO file_entry \
         (entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,trashed_at,created_by,created_at,updated_at) \
         VALUES ($1,$2,NULL,$3,$4,$5,$6,$7,CASE WHEN $7='trashed' THEN CURRENT_TIMESTAMP ELSE NULL END,$8,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .bind(entry_id)
    .bind(SPACE_ID)
    .bind(kind)
    .bind(name)
    .bind(normalized_name)
    .bind(object_id)
    .bind(status)
    .bind(OWNER_ID)
    .execute(pool)
    .await
}

async fn insert_upload_session(pool: &PgPool, session_id: &str, idempotency_key: &str, state: &str) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    query(
        "INSERT INTO file_upload_session \
         (session_id,owner_user_id,space_id,parent_id,idempotency_key,file_name,normalized_name,declared_size_bytes,declared_sha256,content_type,part_size_bytes,provider_key,provider_upload_ref,provider_object_key,state,reserved_bytes,created_at,last_activity_at) \
         VALUES ($1,$2,$3,NULL,$4,'upload.txt','upload.txt',4,repeat('d',64),'text/plain',16777216,'local',$1,$1,$5,4,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .bind(session_id)
    .bind(OWNER_ID)
    .bind(SPACE_ID)
    .bind(idempotency_key)
    .bind(state)
    .execute(pool)
    .await
}
