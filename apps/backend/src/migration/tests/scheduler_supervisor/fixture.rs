mod http;
mod postgres;
mod probe;

use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use http::{HttpFixture, task_context};
use postgres::{advisory_lock_count, advisory_lock_pids, job_command, occurrence_count, schedule_without_notification, wait_for_success};
pub(super) use probe::RuntimeProbe;
use scheduler::{
    application::{
        SchedulerRuntimeConfig, SchedulerRuntimeHandle, SchedulerRuntimeParts, SchedulerService, SchedulerServiceParts, SchedulerUseCase,
        UpdateJobStatusCommand, start_scheduler_runtime,
        task::{ScheduledTaskMetadata, StaticTaskCatalog, TaskExecutionContext},
        tasks::HttpRequestTask,
    },
    domain::JobStatus,
    infra::{PostgresChangeListenerFactory, PostgresExecutionLease, PostgresLeaderLease, StorageSchedulerRepository},
};
use sqlx::{PgPool, postgres::PgPoolOptions, query_scalar};
use storage::Database;
use time::OffsetDateTime;
use tokio::time::{sleep, timeout};

use super::super::TestDatabase;

pub const TAKEOVER_DUE_DELAY_MS: i64 = 3_000;
pub const LOST_NOTIFY_DUE_DELAY_MS: i64 = 0;

const RECONCILE_INTERVAL: Duration = Duration::from_millis(50);
pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(20);
const ASSERT_TIMEOUT: Duration = Duration::from_secs(10);
const REPLICA_POOL_SIZE: u32 = 6;
const REPLICA_COUNT: usize = 2;
const FIRST_REPLICA: usize = 0;
const SECOND_REPLICA: usize = 1;

pub(super) struct OccurrenceTarget {
    pub(super) job_id: String,
    pub(super) revision: i64,
    pub(super) scheduled_at: OffsetDateTime,
}

pub struct SupervisorHarness {
    base_pool: PgPool,
    service: Arc<SchedulerService>,
    replicas: [RuntimeReplica; REPLICA_COUNT],
    http: HttpFixture,
}

impl SupervisorHarness {
    pub async fn start(database: &TestDatabase) -> Self {
        let http = HttpFixture::start().await;
        let catalog = StaticTaskCatalog::try_new([HttpRequestTask::descriptor()]).unwrap();
        let service = scheduler_service(database.pool(), catalog.clone());
        let context = task_context();
        let first = start_replica(ReplicaInput {
            database,
            catalog: catalog.clone(),
            context: context.clone(),
            epoch: "replica-one",
        })
        .await;
        let second = start_replica(ReplicaInput {
            database,
            catalog,
            context,
            epoch: "replica-two",
        })
        .await;
        Self {
            base_pool: database.pool().clone(),
            service,
            replicas: [first, second],
            http,
        }
    }

    pub async fn import_scheduled_job(&self, name: &str, delay_ms: i64) -> OccurrenceTarget {
        let job_id = self.import_paused_job(name).await;
        self.schedule_job_without_notification(&job_id, delay_ms).await
    }

    pub async fn import_paused_job(&self, name: &str) -> String {
        self.service.import_job(job_command(name, &self.http.url)).await.unwrap().job.id
    }

    pub async fn publish_paused_job_change(&self, job_id: &str) {
        self.service
            .update_job_status(UpdateJobStatusCommand {
                id: job_id.into(),
                status: JobStatus::Paused,
                operator: "scheduler-notification-barrier".into(),
            })
            .await
            .unwrap();
    }

    pub async fn schedule_job_without_notification(&self, job_id: &str, delay_ms: i64) -> OccurrenceTarget {
        schedule_without_notification(&self.base_pool, job_id, delay_ms).await
    }

    pub async fn wait_for_any_leader(&self) -> usize {
        timeout(ASSERT_TIMEOUT, async {
            loop {
                let leaders = self.active_leaders();
                if leaders.len() == 1 {
                    return leaders.into_iter().next().expect("leader collection unexpectedly became empty");
                }
                sleep(POLL_INTERVAL).await;
            }
        })
        .await
        .expect("timed out waiting for exactly one scheduler leader")
    }

    pub async fn wait_for_leader(&self, expected: usize) {
        timeout(ASSERT_TIMEOUT, async {
            loop {
                if self.active_leaders() == [expected] {
                    return;
                }
                sleep(POLL_INTERVAL).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for scheduler replica {expected} to become the sole leader"));
    }

    pub async fn wait_for_advisory_lock_count(&self, expected: i64) {
        timeout(ASSERT_TIMEOUT, async {
            loop {
                if advisory_lock_count(&self.base_pool).await == expected {
                    return;
                }
                sleep(POLL_INTERVAL).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for {expected} scheduler advisory lock"));
    }

    pub async fn wait_for_timer_reconcile(&self, index: usize) {
        let before = self.probe(index).timer_reconciles.load(Ordering::SeqCst);
        timeout(ASSERT_TIMEOUT, async {
            loop {
                if self.probe(index).timer_reconciles.load(Ordering::SeqCst) > before {
                    return;
                }
                sleep(POLL_INTERVAL).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for scheduler replica {index} timer reconcile"));
    }

    pub async fn wait_for_notification_reconcile(&self, index: usize, before: usize) {
        timeout(ASSERT_TIMEOUT, async {
            loop {
                if self.probe(index).notification_reconciles.load(Ordering::SeqCst) > before {
                    return;
                }
                sleep(POLL_INTERVAL).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for scheduler replica {index} notification reconcile"));
    }

    pub async fn terminate_leader_session(&self) {
        let pids = advisory_lock_pids(&self.base_pool).await;
        assert_eq!(pids.len(), 1, "leader must be the only advisory lock holder before the occurrence is due");
        let pid = pids.into_iter().next().expect("leader PID collection unexpectedly became empty");
        let terminated = query_scalar::<_, bool>("SELECT pg_terminate_backend($1)")
            .bind(pid)
            .fetch_one(&self.base_pool)
            .await
            .unwrap();
        assert!(terminated, "PostgreSQL did not terminate the scheduler leader session");
    }

    pub fn shutdown_replica(&self, index: usize) {
        self.replicas[index].handle.shutdown();
    }

    pub const fn successor_of(index: usize) -> usize {
        match index {
            FIRST_REPLICA => SECOND_REPLICA,
            SECOND_REPLICA => FIRST_REPLICA,
            _ => panic!("scheduler replica index is out of range"),
        }
    }

    pub fn probe(&self, index: usize) -> &RuntimeProbe {
        &self.replicas[index].probe
    }

    pub fn http_calls(&self) -> usize {
        self.http.calls.load(Ordering::SeqCst)
    }

    pub async fn wait_for_success(&self, target: &OccurrenceTarget) {
        timeout(ASSERT_TIMEOUT, wait_for_success(&self.base_pool, target))
            .await
            .expect("timed out waiting for a terminal scheduler occurrence");
    }

    pub async fn occurrence_count(&self, target: &OccurrenceTarget) -> i64 {
        occurrence_count(&self.base_pool, target).await
    }

    pub async fn shutdown(self) {
        for replica in &self.replicas {
            replica.handle.shutdown();
        }
        wait_for_no_leader(&self.replicas).await;
        for replica in self.replicas {
            replica.pool.close().await;
        }
        self.http.stop().await;
    }

    fn active_leaders(&self) -> Vec<usize> {
        self.replicas
            .iter()
            .enumerate()
            .filter_map(|(index, replica)| replica.probe.leader.load(Ordering::SeqCst).then_some(index))
            .collect()
    }
}

struct RuntimeReplica {
    handle: SchedulerRuntimeHandle,
    pool: PgPool,
    probe: Arc<RuntimeProbe>,
}

struct ReplicaInput<'a> {
    database: &'a TestDatabase,
    catalog: Arc<StaticTaskCatalog>,
    context: TaskExecutionContext,
    epoch: &'a str,
}

async fn start_replica(input: ReplicaInput<'_>) -> RuntimeReplica {
    let pool = replica_pool(input.database).await;
    let probe = Arc::new(RuntimeProbe::default());
    let repository = Arc::new(StorageSchedulerRepository::new(Database::new(pool.clone())));
    let parts = SchedulerRuntimeParts {
        store: repository,
        catalog: input.catalog,
        task_context: input.context,
        leader_lease: Arc::new(PostgresLeaderLease::new(pool.clone())),
        listener_factory: Arc::new(PostgresChangeListenerFactory::new(pool.clone())),
        execution_lease: Arc::new(PostgresExecutionLease::new(pool.clone())),
        telemetry: probe.clone(),
        executor_epoch: input.epoch.into(),
    };
    let handle = start_scheduler_runtime(
        parts,
        SchedulerRuntimeConfig {
            reconcile_interval: RECONCILE_INTERVAL,
        },
    );
    RuntimeReplica { handle, pool, probe }
}

fn scheduler_service(pool: &PgPool, catalog: Arc<StaticTaskCatalog>) -> Arc<SchedulerService> {
    let repository = Arc::new(StorageSchedulerRepository::new(Database::new(pool.clone())));
    Arc::new(SchedulerService::new(SchedulerServiceParts {
        query: repository.clone(),
        commands: repository.clone(),
        audited_commands: repository.clone(),
        catalog,
        clock: repository,
    }))
}

async fn replica_pool(database: &TestDatabase) -> PgPool {
    let url = database.database_url();
    PgPoolOptions::new().max_connections(REPLICA_POOL_SIZE).connect(&url).await.unwrap()
}

async fn wait_for_no_leader(replicas: &[RuntimeReplica; REPLICA_COUNT]) {
    timeout(ASSERT_TIMEOUT, async {
        loop {
            if replicas.iter().all(|replica| !replica.probe.leader.load(Ordering::SeqCst)) {
                return;
            }
            sleep(POLL_INTERVAL).await;
        }
    })
    .await
    .expect("timed out waiting for scheduler supervisors to stop leading");
}
