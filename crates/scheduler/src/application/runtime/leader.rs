use std::{future::pending, time::Duration};

use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, watch};

use crate::{
    application::{ChangeListener, LeaderSession, SchedulerError, SchedulerResult},
    domain::{Execution, ExecutionOutcome},
};

use super::{SchedulerRuntimeConfig, SchedulerRuntimeParts, planner::Planner, supervisor::log_runtime_error};

pub(super) struct LeaderContext<'a> {
    parts: &'a SchedulerRuntimeParts,
    config: SchedulerRuntimeConfig,
    dispatch: &'a mpsc::UnboundedSender<Execution>,
}

impl<'a> LeaderContext<'a> {
    pub fn new(parts: &'a SchedulerRuntimeParts, config: SchedulerRuntimeConfig, dispatch: &'a mpsc::UnboundedSender<Execution>) -> Self {
        Self { parts, config, dispatch }
    }
}

enum LeaderWake {
    Reconcile(&'static str),
    ListenerFailed(SchedulerError),
    Shutdown,
}

pub async fn run_leader(context: LeaderContext<'_>, shutdown: &mut watch::Receiver<bool>, session: &mut dyn LeaderSession) -> SchedulerResult<()> {
    let mut planner = Planner::new();
    let mut listener = connect_listener(context.parts).await;
    let mut next_due = reconcile(&context, &mut planner, "leader_acquired").await;
    loop {
        let now = context.parts.store.database_now().await?;
        let delay = wake_delay(next_due, now, context.config.reconcile_interval)?;
        let reason = match wait_for_wake(delay, &mut listener, shutdown).await {
            LeaderWake::Reconcile(reason) => reason,
            LeaderWake::ListenerFailed(error) => {
                log_runtime_error("notification", &error, context.parts.telemetry.as_ref());
                listener = None;
                "notification"
            }
            LeaderWake::Shutdown => break,
        };
        ensure_leader_alive(session).await?;
        if listener.is_none() {
            listener = connect_listener(context.parts).await;
        }
        next_due = reconcile(&context, &mut planner, reason).await;
    }
    Ok(())
}

async fn wait_for_wake(delay: Duration, listener: &mut Option<Box<dyn ChangeListener>>, shutdown: &mut watch::Receiver<bool>) -> LeaderWake {
    tokio::select! {
        () = tokio::time::sleep(delay) => LeaderWake::Reconcile("timer"),
        notification = wait_notification(listener) => match notification {
            Ok(()) => LeaderWake::Reconcile("notification"),
            Err(error) => LeaderWake::ListenerFailed(error),
        },
        result = shutdown.changed() => {
            if result.is_err() || *shutdown.borrow() {
                LeaderWake::Shutdown
            } else {
                LeaderWake::Reconcile("shutdown_change")
            }
        }
    }
}

async fn ensure_leader_alive(session: &mut dyn LeaderSession) -> SchedulerResult<()> {
    if session.is_alive().await? {
        return Ok(());
    }
    Err(SchedulerError::Infrastructure("scheduler leader session is no longer alive".into()))
}

async fn reconcile(context: &LeaderContext<'_>, planner: &mut Planner, reason: &'static str) -> Option<DateTime<Utc>> {
    let result = reconcile_all(context, planner).await;
    context.parts.telemetry.reconcile(reason, result.is_ok());
    match result {
        Ok(next_due) => next_due,
        Err(error) => {
            log_runtime_error("reconcile", &error, context.parts.telemetry.as_ref());
            None
        }
    }
}

async fn reconcile_all(context: &LeaderContext<'_>, planner: &mut Planner) -> SchedulerResult<Option<DateTime<Utc>>> {
    let next_due = planner.reconcile(context.parts).await?;
    recover_orphans(context.parts).await?;
    let pending = context.parts.store.pending_executions().await?;
    let running = context.parts.store.running_executions().await?;
    context.parts.telemetry.active_executions(pending.len(), running.len());
    for execution in pending {
        context
            .dispatch
            .send(execution)
            .map_err(|_| SchedulerError::Infrastructure("scheduler execution actor is unavailable".into()))?;
    }
    Ok(next_due)
}

async fn recover_orphans(parts: &SchedulerRuntimeParts) -> SchedulerResult<()> {
    let running = parts.store.running_executions().await?;
    for execution in running {
        if parts.execution_lease.is_owned(&execution.id).await? {
            continue;
        }
        let trigger = execution.trigger;
        let ended_at = parts.store.database_now().await?;
        let interrupted = parts
            .store
            .interrupt_execution(crate::application::InterruptExecutionRequest {
                execution_id: execution.id,
                ended_at,
            })
            .await?;
        if interrupted {
            parts.telemetry.execution(trigger.code(), ExecutionOutcome::Interrupted.code());
        }
    }
    Ok(())
}

async fn connect_listener(parts: &SchedulerRuntimeParts) -> Option<Box<dyn ChangeListener>> {
    match parts.listener_factory.connect().await {
        Ok(listener) => Some(listener),
        Err(error) => {
            log_runtime_error("connect_listener", &error, parts.telemetry.as_ref());
            None
        }
    }
}

async fn wait_notification(listener: &mut Option<Box<dyn ChangeListener>>) -> SchedulerResult<()> {
    match listener {
        Some(listener) => listener.wait().await,
        None => pending().await,
    }
}

fn wake_delay(next_due: Option<DateTime<Utc>>, now: DateTime<Utc>, reconcile_interval: Duration) -> SchedulerResult<Duration> {
    let Some(next_due) = next_due else {
        return Ok(reconcile_interval);
    };
    if next_due <= now {
        return Ok(Duration::ZERO);
    }
    let until_due = next_due
        .signed_duration_since(now)
        .to_std()
        .map_err(|error| SchedulerError::Infrastructure(format!("scheduler wake delay conversion failed: {error}")))?;
    Ok(until_due.min(reconcile_interval))
}
