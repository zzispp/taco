mod fixture;

use std::sync::atomic::Ordering;

use fixture::{LOST_NOTIFY_DUE_DELAY_MS, SupervisorHarness, TAKEOVER_DUE_DELAY_MS};

use super::{TestDatabase, fresh};

#[tokio::test]
async fn two_supervisors_take_over_after_leader_session_loss_without_duplicate_occurrence() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let harness = SupervisorHarness::start(&database).await;
    let occurrence = harness.import_scheduled_job("takeover", TAKEOVER_DUE_DELAY_MS).await;

    let first_leader = harness.wait_for_any_leader().await;
    harness.wait_for_advisory_lock_count(1).await;
    harness.terminate_leader_session().await;
    harness.shutdown_replica(first_leader);

    let successor = SupervisorHarness::successor_of(first_leader);
    harness.wait_for_leader(successor).await;
    harness.wait_for_advisory_lock_count(1).await;
    harness.wait_for_success(&occurrence).await;

    assert_eq!(harness.http_calls(), 1);
    assert_eq!(harness.occurrence_count(&occurrence).await, 1);
    assert_eq!(harness.probe(successor).leadership_acquisitions.load(Ordering::SeqCst), 1);

    harness.shutdown().await;
    database.drop().await;
}

#[tokio::test]
async fn periodic_reconcile_executes_a_job_when_no_notification_is_sent() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let harness = SupervisorHarness::start(&database).await;
    let leader = harness.wait_for_any_leader().await;
    harness.wait_for_timer_reconcile(leader).await;
    let notifications_before = harness.probe(leader).notification_reconciles.load(Ordering::SeqCst);
    let job_id = harness.import_paused_job("lost-notify").await;
    harness.publish_paused_job_change(&job_id).await;
    harness.wait_for_notification_reconcile(leader, notifications_before).await;
    harness.wait_for_timer_reconcile(leader).await;
    let timer_reconciles_before = harness.probe(leader).timer_reconciles.load(Ordering::SeqCst);

    let occurrence = harness.schedule_job_without_notification(&job_id, LOST_NOTIFY_DUE_DELAY_MS).await;
    harness.wait_for_success(&occurrence).await;

    assert_eq!(harness.http_calls(), 1);
    assert_eq!(harness.occurrence_count(&occurrence).await, 1);
    assert!(harness.probe(leader).timer_reconciles.load(Ordering::SeqCst) > timer_reconciles_before);

    harness.shutdown().await;
    database.drop().await;
}
