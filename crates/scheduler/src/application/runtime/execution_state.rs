use std::collections::HashMap;

use tokio::{sync::mpsc, task::JoinHandle, time::Interval};

use crate::{
    application::{SchedulerError, SchedulerResult},
    domain::Execution,
};

use super::{
    SchedulerRuntimeConfig, SchedulerRuntimeParts,
    execution_task::{PendingFinish, TaskCompletion, execute_task, panicked_finish, pending_finish},
    supervisor::log_runtime_error,
};

struct RunningTask {
    execution: Execution,
    handle: Option<JoinHandle<()>>,
}

pub(super) struct ExecutionSessionState {
    completion_tx: mpsc::UnboundedSender<TaskCompletion>,
    pub completion_rx: mpsc::UnboundedReceiver<TaskCompletion>,
    running: HashMap<String, RunningTask>,
    finishing: HashMap<String, PendingFinish>,
    pub interval: Interval,
    pub stopping: bool,
}

impl ExecutionSessionState {
    pub fn new(config: SchedulerRuntimeConfig) -> Self {
        let (completion_tx, completion_rx) = mpsc::unbounded_channel();
        Self {
            completion_tx,
            completion_rx,
            running: HashMap::new(),
            finishing: HashMap::new(),
            interval: tokio::time::interval(config.reconcile_interval),
            stopping: false,
        }
    }

    pub fn drained(&self) -> bool {
        self.running.is_empty() && self.finishing.is_empty()
    }

    pub fn contains_running(&self, execution_id: &str) -> bool {
        self.running.contains_key(execution_id)
    }

    pub fn spawn(&mut self, parts: &SchedulerRuntimeParts, execution: Execution) {
        let task_execution = execution.clone();
        let task_parts = parts.clone();
        let completion_sender = self.completion_tx.clone();
        let handle = tokio::spawn(async move {
            let result = execute_task(&task_parts, &task_execution).await;
            let completion = TaskCompletion {
                execution: task_execution,
                result,
            };
            if completion_sender.send(completion).is_err() {
                let error = SchedulerError::Infrastructure("execution completion receiver is unavailable".into());
                log_runtime_error("send_execution_completion", &error, task_parts.telemetry.as_ref());
            }
        });
        self.running.insert(
            execution.id.clone(),
            RunningTask {
                execution,
                handle: Some(handle),
            },
        );
    }

    pub fn complete(&mut self, completion: TaskCompletion) -> SchedulerResult<()> {
        let id = completion.execution.id.clone();
        if self.running.remove(&id).is_none() {
            return Err(SchedulerError::Infrastructure(format!("completion received for unknown execution: {id}")));
        }
        if self.finishing.insert(id.clone(), pending_finish(completion)).is_some() {
            return Err(SchedulerError::Infrastructure(format!("duplicate completion received for execution: {id}")));
        }
        Ok(())
    }

    pub fn finishing_ids(&self) -> Vec<String> {
        self.finishing.keys().cloned().collect()
    }

    pub fn pending_finish(&mut self, execution_id: &str) -> Option<&mut PendingFinish> {
        self.finishing.get_mut(execution_id)
    }

    pub fn remove_finish(&mut self, execution_id: &str) {
        self.finishing.remove(execution_id);
    }

    pub async fn inspect_finished(&mut self) {
        let finished = self
            .running
            .iter()
            .filter(|(_, task)| task.handle.as_ref().is_some_and(JoinHandle::is_finished))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in finished {
            self.inspect_finished_task(&id).await;
        }
    }

    pub async fn abort_running(self) {
        for task in self.running.into_values() {
            abort_task(task).await;
        }
    }

    async fn inspect_finished_task(&mut self, execution_id: &str) {
        let Some(task) = self.running.get_mut(execution_id) else {
            return;
        };
        let execution = task.execution.clone();
        let Some(handle) = task.handle.take() else {
            return;
        };
        if let Err(error) = handle.await {
            self.running.remove(execution_id);
            self.finishing.insert(execution_id.to_owned(), panicked_finish(execution, error));
        }
    }
}

async fn abort_task(task: RunningTask) {
    let Some(handle) = task.handle else {
        return;
    };
    handle.abort();
    match handle.await {
        Ok(()) => {}
        Err(error) if error.is_cancelled() => {}
        Err(error) => hook_tracing::error_with_fields!(
            "scheduled task failed while stopping after execution session loss",
            &error,
            execution_id = task.execution.id.as_str()
        ),
    }
}
