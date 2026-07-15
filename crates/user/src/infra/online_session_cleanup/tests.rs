use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::sync::Notify;

use super::*;

#[derive(Default)]
struct RecordingCleanup {
    batch_sizes: Mutex<Vec<usize>>,
    calls_changed: Notify,
}

impl RecordingCleanup {
    fn call_count(&self) -> usize {
        self.batch_sizes.lock().unwrap().len()
    }

    fn batch_sizes(&self) -> Vec<usize> {
        self.batch_sizes.lock().unwrap().clone()
    }

    async fn wait_for_calls(&self, expected: usize) {
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let changed = self.calls_changed.notified();
                if self.call_count() >= expected {
                    return;
                }
                changed.await;
            }
        })
        .await
        .unwrap();
    }
}

#[async_trait]
impl OnlineSessionCleanup for RecordingCleanup {
    async fn delete_expired(&self, batch_size: usize) -> AppResult<u64> {
        self.batch_sizes.lock().unwrap().push(batch_size);
        self.calls_changed.notify_waiters();
        Ok(1)
    }
}

#[test]
fn config_rejects_zero_and_out_of_range_values() {
    let zero_interval = config(Duration::ZERO, 1);
    let zero_batch = config(Duration::from_secs(1), 0);

    assert!(zero_interval.validate().is_err());
    assert!(zero_batch.validate().is_err());
    #[cfg(target_pointer_width = "64")]
    assert!(config(Duration::from_secs(1), usize::MAX).validate().is_err());
}

#[tokio::test]
async fn cleanup_once_passes_the_configured_batch_size() {
    let cleanup = RecordingCleanup::default();

    assert_eq!(cleanup_once(&cleanup, 17).await.unwrap(), 1);
    assert_eq!(cleanup.batch_sizes(), vec![17]);
}

#[tokio::test]
async fn runtime_runs_immediately_periodically_and_stops() {
    let cleanup = Arc::new(RecordingCleanup::default());
    let handle = start_online_session_cleanup_runtime(OnlineSessionCleanupRuntimeParts {
        cleanup: cleanup.clone(),
        config: config(Duration::from_millis(10), 23),
    })
    .unwrap();
    cleanup.wait_for_calls(2).await;

    handle.shutdown();
    let calls_after_shutdown = cleanup.call_count();
    tokio::time::sleep(Duration::from_millis(30)).await;

    assert_eq!(calls_after_shutdown, 2);
    assert_eq!(cleanup.call_count(), calls_after_shutdown);
    assert_eq!(cleanup.batch_sizes(), vec![23, 23]);
}

fn config(interval: Duration, batch_size: usize) -> OnlineSessionCleanupConfig {
    OnlineSessionCleanupConfig { interval, batch_size }
}
