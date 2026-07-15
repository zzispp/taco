use std::collections::BTreeMap;

use audit_contract::{AuditOutboxEvent, AuditOutboxRecord, AuditStatus, LoginEventType, SecurityAuditEvent};
use kernel::pagination::CursorPageRequest;
use sqlx::{PgPool, query, query_as, query_scalar};
use storage::Database;
use user::{
    application::{AuditedUserRepository, OnlineSession, OnlineSessionPageRequest, OnlineSessionSearch, OnlineSessionStore, ReplaceUserRecord, UserRepository},
    domain::UserId,
    infra::{StorageOnlineSessionStore, StorageUserRepository},
};

use super::{TestDatabase, managed_table_exists, up};

mod storage_lifecycle;

const SESSION_TABLE: &str = "sys_user_session";
const REQUIRED_INDEXES: &[&str] = &["idx_sys_user_session_user", "idx_sys_user_session_expires_at"];
const NANOS_PER_MILLISECOND: i128 = 1_000_000;
const SESSION_LIFETIME_MILLIS: i64 = 300_000;

#[tokio::test]
async fn user_session_migration_creates_expiring_user_bound_sessions() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();

    assert!(managed_table_exists(database.pool(), SESSION_TABLE).await);
    assert_columns(database.pool()).await;
    assert_indexes(database.pool()).await;
    assert_invalid_sessions_are_rejected(database.pool()).await;
    storage_lifecycle::assert_storage_session_lifecycle(database.pool()).await;
    assert_storage_session_cursor_pagination(database.pool()).await;
    assert_security_writes_revoke_sessions(database.pool()).await;

    database.drop().await;
}

async fn assert_storage_session_cursor_pagination(pool: &PgPool) {
    let store = StorageOnlineSessionStore::new(Database::new(pool.clone()));
    create_cursor_sessions(&store).await;
    let first = store.page_active(session_page_request(None)).await.unwrap();
    let second = store.page_active(session_page_request(first.next_cursor.clone())).await.unwrap();
    let returned = store.page_active(session_page_request(second.previous_cursor.clone())).await.unwrap();

    assert_eq!(first.items.iter().map(|session| session.token_id.as_str()).collect::<Vec<_>>(), ["page-c"]);
    assert!(first.has_next);
    assert!(!first.has_previous);
    assert_eq!(second.items.iter().map(|session| session.token_id.as_str()).collect::<Vec<_>>(), ["page-a"]);
    assert!(!second.has_next);
    assert!(second.has_previous);
    assert_eq!(returned.items.iter().map(|session| session.token_id.as_str()).collect::<Vec<_>>(), ["page-c"]);
    assert!(returned.has_next);
    assert!(!returned.has_previous);
    assert_eq!(store.find_active_by_token("page-c").await.unwrap().unwrap().browser, "Firefox");
    query("DELETE FROM sys_user_session").execute(pool).await.unwrap();
}

async fn create_cursor_sessions(store: &StorageOnlineSessionStore) {
    let base = now_millis();
    for (token, browser, age) in [("page-a", "Firefox", 3), ("page-b", "Chrome", 2), ("page-c", "Firefox", 1)] {
        let mut session = active_session(token);
        session.browser = browser.into();
        session.login_time = base - age;
        session.expires_at = base + SESSION_LIFETIME_MILLIS;
        store.create(&session).await.unwrap();
    }
}

fn session_page_request(cursor: Option<String>) -> OnlineSessionPageRequest {
    OnlineSessionPageRequest {
        page: CursorPageRequest { limit: 1, cursor },
        search: OnlineSessionSearch {
            browser: Some("fire".into()),
            ..Default::default()
        },
        scope: None,
    }
}

async fn assert_security_writes_revoke_sessions(pool: &PgPool) {
    let database = Database::new(pool.clone());
    let sessions = StorageOnlineSessionStore::new(database.clone());
    let users = StorageUserRepository::new(database);
    let user_id = test_user_id();

    sessions.create(&active_session("password-session")).await.unwrap();
    users.update_password(user_id.clone(), "new-password-hash".into()).await.unwrap();
    assert_session_count(pool, 0).await;

    sessions.create(&active_session("status-session")).await.unwrap();
    users.update_status(user_id.clone(), "1".into()).await.unwrap();
    assert_session_count(pool, 0).await;
    set_user_status(pool, "0").await;

    sessions.create(&active_session("replace-session")).await.unwrap();
    users.replace(user_id.clone(), replacement_user()).await.unwrap();
    assert_session_count(pool, 0).await;

    sessions.create(&active_session("rollback-session")).await.unwrap();
    let duplicate_audit = duplicate_audit_record();
    insert_existing_outbox_record(pool, &duplicate_audit).await;
    let result = users
        .update_password_with_audit(user_id.clone(), "rolled-back-hash".into(), &duplicate_audit)
        .await;
    assert!(result.is_err());
    assert_session_count(pool, 1).await;
    let password = query_scalar::<_, String>("SELECT password FROM sys_user WHERE user_id=$1")
        .bind(&user_id.0)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(password, "replacement-password-hash");

    sessions.create(&active_session("delete-session")).await.unwrap();
    users.delete(user_id).await.unwrap();
    assert_session_count(pool, 0).await;
}

fn duplicate_audit_record() -> AuditOutboxRecord {
    AuditOutboxRecord {
        id: "duplicate-session-audit".into(),
        occurred_at: time::OffsetDateTime::now_utc(),
        event: AuditOutboxEvent::Security(SecurityAuditEvent {
            request_id: "session-request".into(),
            route: "/api/system/users/session-test-user/password".into(),
            user_id: Some("session-test-user".into()),
            username: "session-test-user".into(),
            ip_address: "127.0.0.1".into(),
            browser: "Test".into(),
            os: "Test".into(),
            status: AuditStatus::Failure,
            event_type: LoginEventType::LoginFailure,
            message_key: "errors.user.invalid_credentials".into(),
            message_params: BTreeMap::new(),
        }),
    }
}

async fn insert_existing_outbox_record(pool: &PgPool, record: &AuditOutboxRecord) {
    query(
        "INSERT INTO audit_outbox (outbox_id,stream,event_type,payload_version,payload,occurred_at) \
         VALUES ($1,'security','login_failure',1,'{}',CURRENT_TIMESTAMP)",
    )
    .bind(&record.id)
    .execute(pool)
    .await
    .unwrap();
}

fn active_session(token_id: &str) -> OnlineSession {
    let login_time = now_millis();
    OnlineSession {
        token_id: token_id.into(),
        user_id: test_user_id(),
        dept_id: None,
        dept_name: None,
        user_name: "session-test-user".into(),
        ipaddr: "127.0.0.1".into(),
        login_location: "Local".into(),
        browser: "Test".into(),
        os: "Test".into(),
        login_time,
        expires_at: login_time + SESSION_LIFETIME_MILLIS,
    }
}

fn replacement_user() -> ReplaceUserRecord {
    ReplaceUserRecord {
        username: "session-test-user".into(),
        password_hash: Some("replacement-password-hash".into()),
        nick_name: "Session Test".into(),
        dept_id: None,
        email: "session@test.invalid".into(),
        phonenumber: None,
        sex: "2".into(),
        status: "0".into(),
        remark: None,
        role_ids: Vec::new(),
        post_ids: Vec::new(),
    }
}

fn test_user_id() -> UserId {
    UserId("session-test-user".into())
}

fn now_millis() -> i64 {
    i64::try_from(time::OffsetDateTime::now_utc().unix_timestamp_nanos() / NANOS_PER_MILLISECOND).unwrap()
}

async fn set_user_status(pool: &PgPool, status: &str) {
    query("UPDATE sys_user SET status=$2,del_flag='0' WHERE user_id=$1")
        .bind(&test_user_id().0)
        .bind(status)
        .execute(pool)
        .await
        .unwrap();
}

async fn assert_session_count(pool: &PgPool, expected: i64) {
    let count = query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_user_session").fetch_one(pool).await.unwrap();
    assert_eq!(count, expected);
}

async fn assert_columns(pool: &PgPool) {
    let columns = query_as::<_, (String, String, String)>(
        "SELECT column_name,data_type,is_nullable FROM information_schema.columns WHERE table_schema='public' AND table_name=$1 ORDER BY ordinal_position",
    )
    .bind(SESSION_TABLE)
    .fetch_all(pool)
    .await
    .unwrap();

    assert_eq!(
        columns,
        vec![
            text_column("token_id", "NO"),
            text_column("user_id", "NO"),
            text_column("dept_name", "YES"),
            text_column("user_name", "NO"),
            text_column("ipaddr", "NO"),
            text_column("login_location", "NO"),
            text_column("browser", "NO"),
            text_column("os", "NO"),
            timestamp_column("login_time"),
            timestamp_column("expires_at"),
        ]
    );
}

async fn assert_indexes(pool: &PgPool) {
    for index in REQUIRED_INDEXES {
        let count = query_scalar::<_, i64>("SELECT COUNT(*) FROM pg_indexes WHERE schemaname='public' AND tablename=$1 AND indexname=$2")
            .bind(SESSION_TABLE)
            .bind(index)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "missing index {index}");
    }
}

async fn assert_invalid_sessions_are_rejected(pool: &PgPool) {
    query(
        "INSERT INTO sys_user (user_id,user_name,nick_name,email,password,create_time) VALUES ('session-test-user','session-test-user','Session Test','session@test.invalid','hash',CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await
    .unwrap();
    let missing_user = query(
        "INSERT INTO sys_user_session (token_id,user_id,user_name,ipaddr,login_location,browser,os,login_time,expires_at) VALUES ('missing','missing','x','x','x','x','x',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP + INTERVAL '1 hour')",
    )
    .execute(pool)
    .await;
    let invalid_expiry = query(
        "INSERT INTO sys_user_session (token_id,user_id,user_name,ipaddr,login_location,browser,os,login_time,expires_at) VALUES ('expired','session-test-user','session-test-user','x','x','x','x',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await;

    assert!(missing_user.is_err());
    assert!(invalid_expiry.is_err());
}

fn text_column(name: &str, nullable: &str) -> (String, String, String) {
    (name.into(), "character varying".into(), nullable.into())
}

fn timestamp_column(name: &str) -> (String, String, String) {
    (name.into(), "timestamp with time zone".into(), "NO".into())
}
