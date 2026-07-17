use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{
    application::{
        OccurrenceAction, OccurrenceRequest, RuntimeErrorUpdate, ScheduleInitialization, SchedulerError, SchedulerResult, SchedulerRuntimeParts,
        next_time_after,
    },
    domain::{Job, MisfirePolicy, RuntimeErrorCode, TriggerType},
};

use super::supervisor::log_runtime_error;

const MILLISECONDS_PER_SECOND: f64 = 1_000.0;

enum ReconcileFailure {
    Deterministic { code: RuntimeErrorCode, source: SchedulerError },
    Infrastructure(SchedulerError),
}

#[derive(Clone, Copy)]
struct TimerEntry {
    revision: i64,
    due_at: DateTime<Utc>,
}

impl ReconcileFailure {
    fn deterministic(code: RuntimeErrorCode, source: SchedulerError) -> Self {
        Self::Deterministic { code, source }
    }

    fn infrastructure(source: SchedulerError) -> Self {
        Self::Infrastructure(source)
    }
}

pub struct Planner {
    timers: HashMap<String, TimerEntry>,
}

impl Planner {
    pub fn new() -> Self {
        Self { timers: HashMap::new() }
    }

    pub async fn reconcile(&mut self, parts: &SchedulerRuntimeParts) -> SchedulerResult<Option<DateTime<Utc>>> {
        let now = parts.store.database_now().await?;
        let jobs = parts.store.schedulable_jobs().await?;
        let active_revisions = jobs.iter().map(|job| (job.id.clone(), job.schedule_revision)).collect::<HashMap<_, _>>();
        self.timers.retain(|job_id, timer| active_revisions.get(job_id) == Some(&timer.revision));
        let mut first_failure = None;
        for job in jobs {
            if let Err(error) = self.reconcile_job(parts, job, now).await {
                log_runtime_error("reconcile_job", &error, parts.telemetry.as_ref());
                if first_failure.is_none() {
                    first_failure = Some(error);
                }
            }
        }
        if let Some(error) = first_failure {
            return Err(error);
        }
        Ok(self.timers.values().map(|timer| timer.due_at).min())
    }

    async fn reconcile_job(&mut self, parts: &SchedulerRuntimeParts, job: Job, now: DateTime<Utc>) -> SchedulerResult<()> {
        if job.runtime_error.is_some() {
            self.timers.remove(&job.id);
            return Ok(());
        }
        match self.try_reconcile_job(parts, &job, now).await {
            Ok(()) => Ok(()),
            Err(ReconcileFailure::Infrastructure(error)) => {
                self.timers.remove(&job.id);
                Err(error)
            }
            Err(ReconcileFailure::Deterministic { code, source }) => {
                log_runtime_error("reconcile_job", &source, parts.telemetry.as_ref());
                let request = runtime_error_update(&job, now, code);
                self.timers.remove(&job.id);
                store_runtime_error(parts, request).await
            }
        }
    }

    async fn try_reconcile_job(&mut self, parts: &SchedulerRuntimeParts, job: &Job, now: DateTime<Utc>) -> Result<(), ReconcileFailure> {
        ensure_runtime_job(parts, job)?;
        let Some(due_at) = job.next_run_at else {
            return self.initialize(parts, job, now).await;
        };
        if due_at > now {
            self.set_timer(job, due_at);
            return Ok(());
        }
        let next_run_at = next_time_after(&job.cron_expression, now).map_err(cron_failure)?;
        let action = self.occurrence_action(job, due_at);
        record_schedule_lag(parts, due_at, now);
        let result = parts
            .store
            .materialize_occurrence(OccurrenceRequest {
                job_id: job.id.clone(),
                expected_revision: job.schedule_revision,
                expected_due_at: due_at,
                next_run_at,
                action,
            })
            .await
            .map_err(ReconcileFailure::infrastructure)?;
        match result {
            crate::application::OccurrenceResult::Materialized | crate::application::OccurrenceResult::AlreadyMaterialized => {
                self.set_timer(job, next_run_at);
            }
            crate::application::OccurrenceResult::Stale => {
                self.timers.remove(&job.id);
            }
        }
        Ok(())
    }

    async fn initialize(&mut self, parts: &SchedulerRuntimeParts, job: &Job, now: DateTime<Utc>) -> Result<(), ReconcileFailure> {
        let next_run_at = next_time_after(&job.cron_expression, now).map_err(cron_failure)?;
        let initialized = parts
            .store
            .initialize_schedule(ScheduleInitialization {
                job_id: job.id.clone(),
                expected_revision: job.schedule_revision,
                next_run_at,
            })
            .await
            .map_err(ReconcileFailure::infrastructure)?;
        if initialized {
            self.set_timer(job, next_run_at);
        } else {
            self.timers.remove(&job.id);
        }
        Ok(())
    }

    fn occurrence_action(&self, job: &Job, due_at: DateTime<Utc>) -> OccurrenceAction {
        if self.timer_matches(job, due_at) {
            return OccurrenceAction::Queue(TriggerType::Scheduled);
        }
        misfire_action(job.misfire_policy)
    }

    fn set_timer(&mut self, job: &Job, due_at: DateTime<Utc>) {
        self.timers.insert(
            job.id.clone(),
            TimerEntry {
                revision: job.schedule_revision,
                due_at,
            },
        );
    }

    fn timer_matches(&self, job: &Job, due_at: DateTime<Utc>) -> bool {
        self.timers
            .get(&job.id)
            .is_some_and(|timer| timer.revision == job.schedule_revision && timer.due_at == due_at)
    }
}

fn misfire_action(policy: MisfirePolicy) -> OccurrenceAction {
    match policy {
        MisfirePolicy::FireOnce => OccurrenceAction::Queue(TriggerType::Misfire),
        MisfirePolicy::DoNothing => OccurrenceAction::SkipMisfire,
    }
}

fn ensure_runtime_job(parts: &SchedulerRuntimeParts, job: &Job) -> Result<(), ReconcileFailure> {
    let definition = parts
        .catalog
        .get(&job.task_key)
        .ok_or_else(|| deterministic_input(RuntimeErrorCode::TaskMissing, "errors.scheduler.task_missing"))?;
    if definition.repeatable != job.repeatable {
        return Err(deterministic_input(
            RuntimeErrorCode::RepeatableMismatch,
            "errors.scheduler.repeatable_mismatch",
        ));
    }
    if definition.params.schema_version != job.params_schema_version {
        return Err(deterministic_input(RuntimeErrorCode::InvalidParams, "errors.scheduler.invalid_params"));
    }
    (definition.params.validate_persisted)(&job.task_params).map_err(|error| ReconcileFailure::deterministic(RuntimeErrorCode::InvalidParams, error))
}

fn deterministic_input(code: RuntimeErrorCode, key: &'static str) -> ReconcileFailure {
    ReconcileFailure::deterministic(code, SchedulerError::InvalidInput(super::super::error::localized(key)))
}

fn runtime_error_update(job: &Job, occurred_at: DateTime<Utc>, code: RuntimeErrorCode) -> RuntimeErrorUpdate {
    RuntimeErrorUpdate {
        job_id: job.id.clone(),
        expected_revision: job.schedule_revision,
        code,
        occurred_at,
    }
}

async fn store_runtime_error(parts: &SchedulerRuntimeParts, request: RuntimeErrorUpdate) -> SchedulerResult<()> {
    parts.store.set_runtime_error(request).await
}

fn cron_failure(error: SchedulerError) -> ReconcileFailure {
    match error {
        SchedulerError::InvalidInput(_) => ReconcileFailure::deterministic(RuntimeErrorCode::InvalidCron, error),
        other => ReconcileFailure::infrastructure(other),
    }
}

fn record_schedule_lag(parts: &SchedulerRuntimeParts, due_at: DateTime<Utc>, now: DateTime<Utc>) {
    let milliseconds = now.signed_duration_since(due_at).num_milliseconds().max(0);
    parts.telemetry.schedule_lag(milliseconds as f64 / MILLISECONDS_PER_SECOND);
}

#[cfg(test)]
mod tests {
    use crate::{
        application::OccurrenceAction,
        domain::{MisfirePolicy, TriggerType},
    };

    use super::misfire_action;

    #[test]
    fn fire_once_materializes_one_misfire_execution() {
        match misfire_action(MisfirePolicy::FireOnce) {
            OccurrenceAction::Queue(TriggerType::Misfire) => {}
            other => panic!("expected one queued misfire execution, got {other:?}"),
        }
    }

    #[test]
    fn do_nothing_materializes_one_skipped_misfire() {
        match misfire_action(MisfirePolicy::DoNothing) {
            OccurrenceAction::SkipMisfire => {}
            other => panic!("expected one skipped misfire occurrence, got {other:?}"),
        }
    }
}
