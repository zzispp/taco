use sqlx::{PgPool, query, query_scalar};
use storage::Database;
use user::{
    application::{OnlineSessionCleanup, OnlineSessionStore},
    infra::StorageOnlineSessionStore,
};

use super::{active_session, set_user_status};

const SESSION_RENEWAL_MILLIS: i64 = 60_000;

pub(super) async fn assert_storage_session_lifecycle(pool: &PgPool) {
    let store = StorageOnlineSessionStore::new(Database::new(pool.clone()));
    insert_expired_session(
        pool,
        ExpiredSessionFixture {
            token_id: "expired-oldest",
            login_age: "2 hours",
            expiry_age: "90 minutes",
        },
    )
    .await;
    insert_expired_session(
        pool,
        ExpiredSessionFixture {
            token_id: "expired-newest",
            login_age: "1 hour",
            expiry_age: "30 minutes",
        },
    )
    .await;
    let session = active_session("session-lifecycle");
    store.create(&session).await.unwrap();

    assert_eq!(session_exists(pool, "expired-oldest").await, 1);
    assert_eq!(store.delete_expired(1).await.unwrap(), 1);
    assert_eq!(session_exists(pool, "expired-oldest").await, 0);
    assert_eq!(session_exists(pool, "expired-newest").await, 1);
    assert_eq!(session_exists(pool, &session.token_id).await, 1);
    assert_eq!(store.delete_expired(1).await.unwrap(), 1);
    assert_eq!(store.delete_expired(1).await.unwrap(), 0);

    let found = store.find_active(&session.token_id, &session.user_id).await.unwrap().unwrap();
    assert_eq!(found, session);
    let renewed_expiry = session.expires_at + SESSION_RENEWAL_MILLIS;
    let renewed = store.renew_active(&session.token_id, &session.user_id, renewed_expiry).await.unwrap().unwrap();
    assert_eq!(renewed.expires_at, renewed_expiry);

    set_user_status(pool, "1").await;
    assert_eq!(store.find_active(&session.token_id, &session.user_id).await.unwrap(), None);
    assert_eq!(
        store
            .renew_active(&session.token_id, &session.user_id, renewed_expiry + SESSION_RENEWAL_MILLIS)
            .await
            .unwrap(),
        None
    );
    set_user_status(pool, "0").await;
    store.delete(&session.token_id).await.unwrap();
}

struct ExpiredSessionFixture<'a> {
    token_id: &'a str,
    login_age: &'a str,
    expiry_age: &'a str,
}

async fn insert_expired_session(pool: &PgPool, fixture: ExpiredSessionFixture<'_>) {
    query(
        "INSERT INTO sys_user_session (token_id,user_id,user_name,ipaddr,login_location,browser,os,login_time,expires_at) \
         VALUES ($1,'session-test-user','session-test-user','x','x','x','x',CURRENT_TIMESTAMP - $2::INTERVAL,CURRENT_TIMESTAMP - $3::INTERVAL)",
    )
    .bind(fixture.token_id)
    .bind(fixture.login_age)
    .bind(fixture.expiry_age)
    .execute(pool)
    .await
    .unwrap();
}

async fn session_exists(pool: &PgPool, token_id: &str) -> i64 {
    query_scalar::<_, i64>("SELECT COUNT(*) FROM sys_user_session WHERE token_id=$1")
        .bind(token_id)
        .fetch_one(pool)
        .await
        .unwrap()
}
