use std::{sync::Arc, time::Duration};

use tokio::sync::watch;

use crate::application::{AppError, AppResult, OnlineSessionCleanup};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OnlineSessionCleanupConfig {
    pub interval: Duration,
    pub batch_size: usize,
}

impl OnlineSessionCleanupConfig {
    fn validate(self) -> AppResult<Self> {
        if self.interval.is_zero() {
            return Err(AppError::Infrastructure("online session cleanup interval must be positive".into()));
        }
        if self.batch_size == 0 {
            return Err(AppError::Infrastructure("online session cleanup batch size must be positive".into()));
        }
        if i64::try_from(self.batch_size).is_err() {
            return Err(AppError::Infrastructure("online session cleanup batch size exceeds the supported range".into()));
        }
        Ok(self)
    }
}

pub struct OnlineSessionCleanupRuntimeParts {
    pub cleanup: Arc<dyn OnlineSessionCleanup>,
    pub config: OnlineSessionCleanupConfig,
}

#[must_use = "retain this handle while expired online sessions must be cleaned"]
#[derive(Clone)]
pub struct OnlineSessionCleanupRuntimeHandle {
    shutdown: watch::Sender<bool>,
}

impl OnlineSessionCleanupRuntimeHandle {
    pub fn shutdown(&self) {
        self.shutdown.send_replace(true);
    }
}

pub fn start_online_session_cleanup_runtime(parts: OnlineSessionCleanupRuntimeParts) -> AppResult<OnlineSessionCleanupRuntimeHandle> {
    let config = parts.config.validate()?;
    let (shutdown, receiver) = watch::channel(false);
    tokio::spawn(run_cleanup(parts.cleanup, config, receiver));
    Ok(OnlineSessionCleanupRuntimeHandle { shutdown })
}

async fn run_cleanup(cleanup: Arc<dyn OnlineSessionCleanup>, config: OnlineSessionCleanupConfig, mut shutdown: watch::Receiver<bool>) {
    loop {
        if shutdown_requested(&shutdown) {
            return;
        }
        if let Err(error) = cleanup_once(cleanup.as_ref(), config.batch_size).await {
            hook_tracing::error_with_fields!("online session cleanup failed", &error, reason = "cleanup_failed");
        }
        if wait_for(config.interval, &mut shutdown).await {
            return;
        }
    }
}

async fn cleanup_once(cleanup: &dyn OnlineSessionCleanup, batch_size: usize) -> AppResult<u64> {
    cleanup.delete_expired(batch_size).await
}

async fn wait_for(delay: Duration, shutdown: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        () = tokio::time::sleep(delay) => false,
        result = shutdown.changed() => result.is_err() || *shutdown.borrow(),
    }
}

fn shutdown_requested(shutdown: &watch::Receiver<bool>) -> bool {
    *shutdown.borrow() || shutdown.has_changed().is_err()
}

#[cfg(test)]
mod tests;
