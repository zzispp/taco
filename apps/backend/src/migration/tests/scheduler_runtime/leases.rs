use scheduler::{
    application::{ExecutionLease, LeaderLease, SchedulerError},
    infra::{PostgresExecutionLease, PostgresLeaderLease},
};
use sqlx::{PgPool, query_scalar};

use super::super::{TestDatabase, fresh};

const EXECUTION_ID: &str = "execution-lease-test";
const SECOND_EXECUTION_ID: &str = "execution-lease-test-two";

#[tokio::test]
async fn postgres_leases_enforce_single_ownership_and_allow_takeover() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();

    assert_leader_release_takeover(database.pool()).await;
    assert_leader_session_loss_takeover(database.pool()).await;
    assert_execution_lease_ownership(database.pool()).await;

    database.drop().await;
}

async fn assert_leader_release_takeover(pool: &PgPool) {
    let first = PostgresLeaderLease::new(pool.clone());
    let second = PostgresLeaderLease::new(pool.clone());
    let mut leader = first.try_acquire().await.unwrap().unwrap();

    assert!(leader.is_alive().await.unwrap());
    assert!(second.try_acquire().await.unwrap().is_none());
    leader.release().await.unwrap();

    let mut successor = second.try_acquire().await.unwrap().unwrap();
    assert!(successor.is_alive().await.unwrap());
    successor.release().await.unwrap();
}

async fn assert_leader_session_loss_takeover(pool: &PgPool) {
    let first = PostgresLeaderLease::new(pool.clone());
    let second = PostgresLeaderLease::new(pool.clone());
    let mut leader = first.try_acquire().await.unwrap().unwrap();

    terminate_advisory_lock_holder(pool).await;
    assert!(matches!(leader.is_alive().await, Err(SchedulerError::Infrastructure(_))));
    drop(leader);

    let mut successor = second.try_acquire().await.unwrap().unwrap();
    assert!(successor.is_alive().await.unwrap());
    successor.release().await.unwrap();
}

async fn terminate_advisory_lock_holder(pool: &PgPool) {
    let pid = query_scalar::<_, i32>(
        "SELECT pid FROM pg_locks WHERE locktype='advisory' AND database=(SELECT oid FROM pg_database WHERE datname=current_database()) AND granted",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let terminated = query_scalar::<_, bool>("SELECT pg_terminate_backend($1)")
        .bind(pid)
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(terminated);
}

async fn assert_execution_lease_ownership(pool: &PgPool) {
    let lease = PostgresExecutionLease::new(pool.clone());
    let mut session = lease.open_session().await.unwrap();

    assert!(session.try_acquire(EXECUTION_ID).await.unwrap());
    assert!(lease.is_owned(EXECUTION_ID).await.unwrap());
    session.release(EXECUTION_ID).await.unwrap();
    assert!(!lease.is_owned(EXECUTION_ID).await.unwrap());

    assert!(session.try_acquire(EXECUTION_ID).await.unwrap());
    assert!(session.try_acquire(SECOND_EXECUTION_ID).await.unwrap());
    session.release_all().await.unwrap();
    assert!(!lease.is_owned(EXECUTION_ID).await.unwrap());
    assert!(!lease.is_owned(SECOND_EXECUTION_ID).await.unwrap());
}
