use sqlx::{PgPool, query_as, query_scalar};

const FILE_PERMISSIONS: &[&str] = &[
    "file:asset:download",
    "file:asset:edit",
    "file:asset:list",
    "file:asset:purge",
    "file:asset:query",
    "file:asset:remove",
    "file:asset:restore",
    "file:asset:upload",
    "file:folder:add",
    "file:provider:query",
    "file:space:list",
    "file:space:quota",
    "file:upload:manage",
];

pub(super) async fn assert_file_management_seed(pool: &PgPool) {
    assert_pages(pool).await;
    assert_permissions(pool).await;
    assert_local_provider(pool).await;
    assert_cleanup_jobs(pool).await;
    assert_avatar_contract(pool).await;
}

async fn assert_pages(pool: &PgPool) {
    let pages: Vec<(String, String, String, String)> = query_as(
        "SELECT menu_id,parent_id,path,COALESCE(perms,'') FROM sys_menu \
         WHERE menu_id IN ('5','115','116','117') ORDER BY menu_id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(
        pages,
        vec![
            ("115".into(), "5".into(), "/dashboard/file".into(), "file:asset:list".into()),
            ("116".into(), "5".into(), "/dashboard/file-manager".into(), "file:asset:list".into()),
            ("117".into(), "5".into(), "/dashboard/file-spaces".into(), "file:space:list".into()),
            ("5".into(), "0".into(), "/dashboard/files".into(), "".into()),
        ]
    );
}

async fn assert_permissions(pool: &PgPool) {
    let permissions: Vec<String> = query_scalar("SELECT DISTINCT perms FROM sys_menu WHERE perms LIKE 'file:%' ORDER BY perms")
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(permissions, FILE_PERMISSIONS.iter().map(|value| (*value).into()).collect::<Vec<String>>());
}

async fn assert_local_provider(pool: &PgPool) {
    let provider: (String, String, String) = query_as("SELECT provider_key,provider_type,status FROM file_storage_provider WHERE provider_key='local'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(provider, ("local".into(), "local".into(), "active".into()));
}

async fn assert_cleanup_jobs(pool: &PgPool) {
    let jobs: Vec<(String, String, String, String, String, String)> = query_as(
        "SELECT job_id,task_key,cron_expression,misfire_policy,concurrent,status FROM sys_job \
         WHERE job_id IN ('file-purge-trash','file-cleanup-upload-sessions') ORDER BY job_id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(
        jobs,
        vec![
            (
                "file-cleanup-upload-sessions".into(),
                "file.cleanupUploadSessions".into(),
                "0 0 21 * * *".into(),
                "2".into(),
                "1".into(),
                "0".into(),
            ),
            (
                "file-purge-trash".into(),
                "file.purgeTrash".into(),
                "0 0 20 * * *".into(),
                "2".into(),
                "1".into(),
                "0".into(),
            ),
        ]
    );
}

async fn assert_avatar_contract(pool: &PgPool) {
    let columns: Vec<String> = query_scalar(
        "SELECT column_name FROM information_schema.columns WHERE table_schema='public' AND table_name='sys_user' \
         AND column_name IN ('avatar','avatar_file_id','avatar_version') ORDER BY column_name",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(columns, vec!["avatar_file_id".to_owned(), "avatar_version".to_owned()]);
}
