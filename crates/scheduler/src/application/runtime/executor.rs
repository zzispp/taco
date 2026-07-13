use tokio::sync::{mpsc, watch};

use crate::{
    application::{ClaimExecutionRequest, ExecutionLeaseSession, SchedulerError, SchedulerResult},
    domain::Execution,
};

use super::{
    SchedulerRuntimeConfig, SchedulerRuntimeParts,
    execution_state::ExecutionSessionState,
    execution_task::{PendingFinish, TaskCompletion},
    supervisor::log_runtime_error,
};

enum ActorEvent {
    Dispatch(Option<Execution>),
    Completion(Option<TaskCompletion>),
    Tick,
    Shutdown(Result<(), watch::error::RecvError>),
}

pub(super) struct ExecutionActorChannels {
    executions: mpsc::UnboundedReceiver<Execution>,
    shutdown: watch::Receiver<bool>,
}

impl ExecutionActorChannels {
    pub fn new(executions: mpsc::UnboundedReceiver<Execution>, shutdown: watch::Receiver<bool>) -> Self {
        Self { executions, shutdown }
    }
}

pub struct ExecutionActor {
    parts: SchedulerRuntimeParts,
    config: SchedulerRuntimeConfig,
    channels: ExecutionActorChannels,
}

impl ExecutionActor {
    pub(super) fn new(parts: SchedulerRuntimeParts, config: SchedulerRuntimeConfig, channels: ExecutionActorChannels) -> Self {
        Self { parts, config, channels }
    }

    pub async fn run(mut self) {
        while !self.stopping() {
            match self.parts.execution_lease.open_session().await {
                Ok(mut session) => {
                    if let Err(error) = self.run_session(session.as_mut()).await {
                        log_runtime_error("execution_session", &error, self.parts.telemetry.as_ref());
                    }
                }
                Err(error) => log_runtime_error("open_execution_session", &error, self.parts.telemetry.as_ref()),
            }
            if !self.stopping() {
                self.wait_retry().await;
            }
        }
    }

    async fn run_session(&mut self, session: &mut dyn ExecutionLeaseSession) -> SchedulerResult<()> {
        let mut state = ExecutionSessionState::new(self.config);
        let result = self.drive_session(session, &mut state).await;
        if result.is_err() {
            state.abort_running().await;
        }
        result
    }

    async fn drive_session(&mut self, session: &mut dyn ExecutionLeaseSession, state: &mut ExecutionSessionState) -> SchedulerResult<()> {
        loop {
            let event = self.next_event(state).await;
            self.handle_event(session, state, event).await?;
            self.retry_finishing(session, state).await?;
            if state.stopping && state.drained() {
                return session.release_all().await;
            }
        }
    }

    async fn next_event(&mut self, state: &mut ExecutionSessionState) -> ActorEvent {
        tokio::select! {
            execution = self.channels.executions.recv(), if !state.stopping => ActorEvent::Dispatch(execution),
            completion = state.completion_rx.recv() => ActorEvent::Completion(completion),
            _ = state.interval.tick() => ActorEvent::Tick,
            result = self.channels.shutdown.changed() => ActorEvent::Shutdown(result),
        }
    }

    async fn handle_event(&self, session: &mut dyn ExecutionLeaseSession, state: &mut ExecutionSessionState, event: ActorEvent) -> SchedulerResult<()> {
        match event {
            ActorEvent::Dispatch(Some(execution)) => self.start_execution(session, state, execution).await,
            ActorEvent::Dispatch(None) => {
                state.stopping = true;
                Ok(())
            }
            ActorEvent::Completion(Some(completion)) => state.complete(completion),
            ActorEvent::Completion(None) => Err(SchedulerError::Infrastructure("execution completion channel closed unexpectedly".into())),
            ActorEvent::Tick => check_session(session, state).await,
            ActorEvent::Shutdown(result) => {
                state.stopping = result.is_err() || *self.channels.shutdown.borrow();
                Ok(())
            }
        }
    }

    async fn start_execution(&self, session: &mut dyn ExecutionLeaseSession, state: &mut ExecutionSessionState, execution: Execution) -> SchedulerResult<()> {
        if state.contains_running(&execution.id) || !session.try_acquire(&execution.id).await? {
            return Ok(());
        }
        let execution_id = execution.id.clone();
        match self.claim_execution(&execution_id).await {
            Ok(Some(execution)) => {
                state.spawn(&self.parts, execution);
                Ok(())
            }
            Ok(None) => session.release(&execution_id).await,
            Err(error) => self.release_failed_claim(session, &execution_id, error).await,
        }
    }

    async fn claim_execution(&self, execution_id: &str) -> SchedulerResult<Option<Execution>> {
        let started_at = self.parts.store.database_now().await?;
        self.parts
            .store
            .claim_execution(ClaimExecutionRequest {
                execution_id: execution_id.to_owned(),
                executor_epoch: self.parts.executor_epoch.clone(),
                started_at,
            })
            .await
    }

    async fn release_failed_claim(&self, session: &mut dyn ExecutionLeaseSession, execution_id: &str, claim_error: SchedulerError) -> SchedulerResult<()> {
        if let Err(release_error) = session.release(execution_id).await {
            log_runtime_error("claim_execution", &claim_error, self.parts.telemetry.as_ref());
            return Err(release_error);
        }
        log_runtime_error("claim_execution", &claim_error, self.parts.telemetry.as_ref());
        Ok(())
    }

    async fn retry_finishing(&self, session: &mut dyn ExecutionLeaseSession, state: &mut ExecutionSessionState) -> SchedulerResult<()> {
        let ids = state.finishing_ids();
        for id in ids {
            let pending = state.pending_finish(&id).ok_or_else(|| missing_finish(&id))?;
            if let Err(error) = self.finish(pending).await {
                log_runtime_error("finish_execution", &error, self.parts.telemetry.as_ref());
                continue;
            }
            session.release(&id).await?;
            state.remove_finish(&id);
        }
        Ok(())
    }

    async fn finish(&self, pending: &mut PendingFinish) -> SchedulerResult<()> {
        let ended_at = match pending.ended_at() {
            Some(ended_at) => ended_at,
            None => self.parts.store.database_now().await?,
        };
        let request = pending.request(ended_at);
        if !self.parts.store.finish_execution(request).await? {
            return Err(SchedulerError::Infrastructure(format!(
                "execution did not transition to the requested terminal state: {}",
                pending.execution.id
            )));
        }
        self.parts.telemetry.execution(pending.execution.trigger.code(), pending.outcome.code());
        Ok(())
    }

    fn stopping(&self) -> bool {
        *self.channels.shutdown.borrow() || self.channels.shutdown.has_changed().is_err() || self.channels.executions.is_closed()
    }

    async fn wait_retry(&mut self) {
        tokio::select! {
            () = tokio::time::sleep(self.config.reconcile_interval) => {}
            _ = self.channels.shutdown.changed() => {}
        }
    }
}

async fn check_session(session: &mut dyn ExecutionLeaseSession, state: &mut ExecutionSessionState) -> SchedulerResult<()> {
    state.inspect_finished().await;
    if session.is_alive().await? {
        return Ok(());
    }
    Err(SchedulerError::Infrastructure("scheduler execution session is no longer alive".into()))
}

fn missing_finish(execution_id: &str) -> SchedulerError {
    SchedulerError::Infrastructure(format!("pending terminal result disappeared: {execution_id}"))
}
