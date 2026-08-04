use sqlx::{PgPool, query, query_as, query_scalar};

const OWNER_ID: &str = "index-owner";
const SPACE_ID: &str = "index-space";
const ROOT_ENTRY_ID: &str = "index-root";
const CHILD_ENTRY_ID: &str = "index-child";
const CONTENT_ENTRY_ID: &str = "index-content";
const AVATAR_ENTRY_ID: &str = "index-avatar";
const UPLOAD_PARENT_ENTRY_ID: &str = "index-upload-parent";
const UPLOAD_RESULT_ENTRY_ID: &str = "index-upload-result";
const CONTENT_OBJECT_ID: &str = "index-content-object";

pub(super) async fn seed_existing_file_management_data(pool: &PgPool) {
    insert_owner_and_space(pool).await;
    insert_objects(pool).await;
    insert_entries(pool).await;
    insert_tag(pool).await;
    insert_upload_sessions(pool).await;
    bind_avatar(pool).await;
}

pub(super) async fn assert_existing_file_management_data(pool: &PgPool) {
    assert_tree_and_object_data(pool).await;
    assert_reference_data(pool).await;
}

pub(super) async fn assert_reference_integrity(pool: &PgPool) {
    assert_tree_object_and_avatar_deletes_are_restricted(pool).await;
    assert_tag_and_upload_references_keep_their_delete_semantics(pool).await;
}

async fn insert_owner_and_space(pool: &PgPool) {
    query("INSERT INTO sys_user(user_id,user_name,nick_name,password,create_time) VALUES($1,'index-owner','Index Owner','hash',CURRENT_TIMESTAMP)")
        .bind(OWNER_ID)
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO file_space(space_id,owner_user_id,created_at,updated_at) VALUES($1,$2,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)")
        .bind(SPACE_ID)
        .bind(OWNER_ID)
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_objects(pool: &PgPool) {
    query(
        "INSERT INTO file_object(object_id,provider_key,object_key,size_bytes,sha256,content_type,ref_count,status,created_at,updated_at) VALUES \
         ('index-content-object','local','objects/content',4,repeat('a',64),'text/plain',1,'active',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP), \
         ('index-avatar-object','local','objects/avatar',4,repeat('b',64),'image/png',1,'active',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP), \
         ('index-result-object','local','objects/result',4,repeat('c',64),'text/plain',1,'active',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_entries(pool: &PgPool) {
    query(
        "INSERT INTO file_entry(entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,created_by,created_at,updated_at) VALUES \
         ('index-root',$1,NULL,'folder','Root','root',NULL,'active',$2,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP), \
         ('index-child',$1,'index-root','folder','Child','child',NULL,'active',$2,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP), \
         ('index-content',$1,'index-child','file','Document.txt','document.txt','index-content-object','active',$2,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP), \
         ('index-avatar',$1,NULL,'file','Avatar.png','avatar.png','index-avatar-object','active',$2,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP), \
         ('index-upload-parent',$1,NULL,'folder','Uploads','uploads',NULL,'active',$2,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP), \
         ('index-upload-result',$1,NULL,'file','Completed.txt','completed.txt','index-result-object','active',$2,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .bind(SPACE_ID)
    .bind(OWNER_ID)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_tag(pool: &PgPool) {
    query("INSERT INTO file_tag(tag_id,space_id,name,normalized_name,created_by,created_at) VALUES('index-tag',$1,'Indexed','indexed',$2,CURRENT_TIMESTAMP)")
        .bind(SPACE_ID)
        .bind(OWNER_ID)
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO file_entry_tag(entry_id,tag_id,created_at) VALUES($1,'index-tag',CURRENT_TIMESTAMP)")
        .bind(CONTENT_ENTRY_ID)
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_upload_sessions(pool: &PgPool) {
    query(
        "INSERT INTO file_upload_session(session_id,owner_user_id,space_id,parent_id,idempotency_key,file_name,normalized_name,declared_size_bytes,declared_sha256,content_type,part_size_bytes,provider_key,provider_upload_ref,provider_object_key,state,reserved_bytes,created_at,last_activity_at) \
         VALUES('index-open-upload',$1,$2,'index-upload-parent','index-open-intent','open.txt','open.txt',4,repeat('d',64),'text/plain',4,'local','index-open-ref','objects/open','open',4,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .bind(OWNER_ID)
    .bind(SPACE_ID)
    .execute(pool)
    .await
    .unwrap();
    query(
        "INSERT INTO file_upload_session(session_id,owner_user_id,space_id,parent_id,idempotency_key,file_name,normalized_name,declared_size_bytes,declared_sha256,content_type,part_size_bytes,provider_key,provider_upload_ref,provider_object_key,state,reserved_bytes,result_entry_id,created_at,last_activity_at,completed_at) \
         VALUES('index-completed-upload',$1,$2,NULL,'index-completed-intent','completed.txt','completed.txt',4,repeat('e',64),'text/plain',4,'local','index-completed-ref','objects/completed','completed',0,'index-upload-result',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .bind(OWNER_ID)
    .bind(SPACE_ID)
    .execute(pool)
    .await
    .unwrap();
}

async fn bind_avatar(pool: &PgPool) {
    query("UPDATE sys_user SET avatar_file_id=$1,avatar_version=1 WHERE user_id=$2")
        .bind(AVATAR_ENTRY_ID)
        .bind(OWNER_ID)
        .execute(pool)
        .await
        .unwrap();
}

async fn assert_tree_and_object_data(pool: &PgPool) {
    let rows: Vec<(String, Option<String>, Option<String>)> =
        query_as("SELECT entry_id,parent_id,object_id FROM file_entry WHERE entry_id = ANY($1) ORDER BY entry_id")
            .bind(vec![
                ROOT_ENTRY_ID,
                CHILD_ENTRY_ID,
                CONTENT_ENTRY_ID,
                AVATAR_ENTRY_ID,
                UPLOAD_PARENT_ENTRY_ID,
                UPLOAD_RESULT_ENTRY_ID,
            ])
            .fetch_all(pool)
            .await
            .unwrap();
    assert_eq!(
        rows,
        vec![
            (AVATAR_ENTRY_ID.into(), None, Some("index-avatar-object".into())),
            (CHILD_ENTRY_ID.into(), Some(ROOT_ENTRY_ID.into()), None),
            (CONTENT_ENTRY_ID.into(), Some(CHILD_ENTRY_ID.into()), Some(CONTENT_OBJECT_ID.into())),
            (ROOT_ENTRY_ID.into(), None, None),
            (UPLOAD_PARENT_ENTRY_ID.into(), None, None),
            (UPLOAD_RESULT_ENTRY_ID.into(), None, Some("index-result-object".into())),
        ]
    );
}

async fn assert_reference_data(pool: &PgPool) {
    let tag_links: i64 = query_scalar("SELECT COUNT(*) FROM file_entry_tag WHERE entry_id=$1 AND tag_id='index-tag'")
        .bind(CONTENT_ENTRY_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    let sessions: Vec<(String, Option<String>, Option<String>)> = query_as(
        "SELECT session_id,parent_id,result_entry_id FROM file_upload_session WHERE session_id IN ('index-open-upload','index-completed-upload') ORDER BY session_id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    let avatar: Option<String> = query_scalar("SELECT avatar_file_id FROM sys_user WHERE user_id=$1")
        .bind(OWNER_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(tag_links, 1);
    assert_eq!(
        sessions,
        vec![
            ("index-completed-upload".into(), None, Some(UPLOAD_RESULT_ENTRY_ID.into())),
            ("index-open-upload".into(), Some(UPLOAD_PARENT_ENTRY_ID.into()), None),
        ]
    );
    assert_eq!(avatar, Some(AVATAR_ENTRY_ID.into()));
}

async fn assert_tree_object_and_avatar_deletes_are_restricted(pool: &PgPool) {
    let tree_delete = query("DELETE FROM file_entry WHERE entry_id=$1").bind(ROOT_ENTRY_ID).execute(pool).await;
    let object_delete = query("DELETE FROM file_object WHERE object_id=$1").bind(CONTENT_OBJECT_ID).execute(pool).await;
    let avatar_delete = query("DELETE FROM file_entry WHERE entry_id=$1").bind(AVATAR_ENTRY_ID).execute(pool).await;
    assert!(tree_delete.is_err(), "child entries must prevent deleting their parent");
    assert!(object_delete.is_err(), "file entries must prevent deleting their object");
    assert!(avatar_delete.is_err(), "avatar references must prevent deleting their file entry");
}

async fn assert_tag_and_upload_references_keep_their_delete_semantics(pool: &PgPool) {
    query("DELETE FROM file_tag WHERE tag_id='index-tag'").execute(pool).await.unwrap();
    query("DELETE FROM file_entry WHERE entry_id=$1")
        .bind(UPLOAD_PARENT_ENTRY_ID)
        .execute(pool)
        .await
        .unwrap();
    query("DELETE FROM file_entry WHERE entry_id=$1")
        .bind(UPLOAD_RESULT_ENTRY_ID)
        .execute(pool)
        .await
        .unwrap();
    let tag_links: i64 = query_scalar("SELECT COUNT(*) FROM file_entry_tag WHERE entry_id=$1")
        .bind(CONTENT_ENTRY_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    let references: (Option<String>, Option<String>) = query_as(
        "SELECT (SELECT parent_id FROM file_upload_session WHERE session_id='index-open-upload'),(SELECT result_entry_id FROM file_upload_session WHERE session_id='index-completed-upload')",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(tag_links, 0, "deleting a tag must cascade to its entry link");
    assert_eq!(references, (None, None), "upload references must be cleared by their foreign keys");
}
