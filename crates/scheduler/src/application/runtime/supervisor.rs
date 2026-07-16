use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};

use crate::{
    application::{SchedulerError, SchedulerResult, SchedulerTelemetry},
    domain::Execution,
};

use super::{
    SchedulerRuntimeConfig, SchedulerRuntimeHandle, SchedulerRuntimeParts,
    executor::{ExecutionActor, ExecutionActorChannels},
    leader::{LeaderContext, run_leader},
};

struct SupervisorContext {
    parts: SchedulerRuntimeParts,
    config: SchedulerRuntimeConfig,
    dispatch: mpsc::UnboundedSender<Execution>,
}

pub fn start_scheduler_runtime(parts: SchedulerRuntimeParts, config: SchedulerRuntimeConfig) -> SchedulerRuntimeHandle {
    let (shutdown, receiver) = watch::channel(false);
    tokio::spawn(run_supervisor(parts, config, receiver));
    SchedulerRuntimeHandle { shutdown }
}

async fn run_supervisor(parts: SchedulerRuntimeParts, config: SchedulerRuntimeConfig, mut shutdown: watch::Receiver<bool>) {
    let (dispatch, executions) = mpsc::unbounded_channel();
    let channels = ExecutionActorChannels::new(executions, shutdown.clone());
    let actor = spawn_execution_actor(&parts, config, channels);
    let context = SupervisorContext { parts, config, dispatch };
    while !shutdown_requested(&shutdown) {
        if let Err(error) = leadership_cycle(&context, &mut shutdown).await {
            log_runtime_error("leadership_cycle", &error, context.parts.telemetry.as_ref());
        }
        if shutdown_requested(&shutdown) || wait_retry(config, &mut shutdown).await {
            break;
        }
    }
    drop(context);
    join_execution_actor(actor).await;
}

fn spawn_execution_actor(parts: &SchedulerRuntimeParts, config: SchedulerRuntimeConfig, channels: ExecutionActorChannels) -> JoinHandle<()> {
    let actor = ExecutionActor::new(parts.clone(), config, channels);
    tokio::spawn(actor.run())
}

async fn leadership_cycle(context: &SupervisorContext, shutdown: &mut watch::Receiver<bool>) -> SchedulerResult<()> {
    let Some(mut session) = context.parts.leader_lease.try_acquire().await? else {
        return Ok(());
    };
    context.parts.telemetry.leadership(true);
    let leader = LeaderContext::new(&context.parts, context.config, &context.dispatch);
    let run_result = run_leader(leader, shutdown, session.as_mut()).await;
    let release_result = session.release().await;
    context.parts.telemetry.leadership(false);
    merge_leader_results(run_result, release_result, context.parts.telemetry.as_ref())
}

fn merge_leader_results(run_result: SchedulerResult<()>, release_result: SchedulerResult<()>, telemetry: &dyn SchedulerTelemetry) -> SchedulerResult<()> {
    match (run_result, release_result) {
        (Err(run_error), Err(release_error)) => {
            log_runtime_error("release_leader", &release_error, telemetry);
            Err(run_error)
        }
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

async fn wait_retry(config: SchedulerRuntimeConfig, shutdown: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        () = tokio::time::sleep(config.reconcile_interval) => shutdown_requested(shutdown),
        result = shutdown.changed() => result.is_err() || *shutdown.borrow(),
    }
}

fn shutdown_requested(shutdown: &watch::Receiver<bool>) -> bool {
    *shutdown.borrow() || shutdown.has_changed().is_err()
}

async fn join_execution_actor(actor: JoinHandle<()>) {
    if let Err(error) = actor.await {
        taco_tracing::error_with_fields!("scheduler execution actor stopped unexpectedly", &error,);
    }
}

pub(super) fn log_runtime_error(operation: &'static str, error: &SchedulerError, telemetry: &dyn SchedulerTelemetry) {
    telemetry.runtime_error(operation);
    taco_tracing::error_with_fields!("scheduler runtime operation failed", error, operation = operation);
}
