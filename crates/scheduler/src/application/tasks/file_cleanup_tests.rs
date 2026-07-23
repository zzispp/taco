use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;

use crate::application::task::{
    FileCleanupPort, FileTrashCleanupResult, FileUploadSessionCleanupResult, HttpTaskClient, OutboundHttpFailure, OutboundHttpRequest, OutboundHttpResponse,
    ScheduledTask, ScheduledTaskMetadata, SystemCacheRefreshPort, SystemLogCleanupFilter, SystemLogCleanupPort, SystemLogCleanupResult, TaskExecutionContext,
    TaskExecutionFailure, TaskInvocation, TaskLifecyclePolicy,
};

use super::{CleanupUploadSessionsTask, PurgeTrashTask};

#[tokio::test]
async fn purge_trash_passes_retention_policy_and_reports_deleted_content() {
    let cleanup = Arc::new(RecordingFileCleanup::default());

    let output = PurgeTrashTask
        .execute(
            context(cleanup.clone()),
            invocation("file.purgeTrash", json!({"retention_days": 30, "batch_size": 1000})),
        )
        .await
        .unwrap();

    assert_eq!(cleanup.purge_calls.lock().unwrap().as_slice(), &[(30, 1000)]);
    let detail = output.detail.unwrap();
    assert_eq!(detail.kind(), "file_trash_cleanup");
    assert_eq!(detail.schema_version(), 2);
    assert_eq!(
        detail.payload(),
        &json!({"purged_entries": 7, "blocked_roots": 2, "deleted_objects": 5, "failed_objects": 0, "retried_provider_cleanups": 3, "failed_provider_cleanups": 0})
    );
}

#[tokio::test]
async fn upload_session_cleanup_passes_batch_size_and_reports_terminal_sessions() {
    let cleanup = Arc::new(RecordingFileCleanup::default());

    let output = CleanupUploadSessionsTask
        .execute(context(cleanup.clone()), invocation("file.cleanupUploadSessions", json!({"batch_size": 1000})))
        .await
        .unwrap();

    assert_eq!(cleanup.upload_calls.lock().unwrap().as_slice(), &[1000]);
    let detail = output.detail.unwrap();
    assert_eq!(detail.kind(), "file_upload_session_cleanup");
    assert_eq!(detail.schema_version(), 2);
    assert_eq!(
        detail.payload(),
        &json!({"expired_sessions": 4, "reconciled_sessions": 2, "retried_provider_cleanups": 1, "failed_provider_cleanups": 0})
    );
}

#[tokio::test]
async fn task_rejects_invalid_parameters_before_calling_cleanup_port() {
    let cleanup = Arc::new(RecordingFileCleanup::default());

    let failure = PurgeTrashTask
        .execute(
            context(cleanup.clone()),
            invocation("file.purgeTrash", json!({"retention_days": 0, "batch_size": 1000})),
        )
        .await
        .unwrap_err();

    assert_eq!(failure.public.key(), "errors.scheduler.invalid_params");
    assert_eq!(cleanup.purge_calls.lock().unwrap().as_slice(), &[]);
}

#[test]
fn file_cleanup_tasks_are_required_but_pausable() {
    for definition in [PurgeTrashTask::descriptor(), CleanupUploadSessionsTask::descriptor()] {
        assert_eq!(definition.lifecycle, TaskLifecyclePolicy::RequiredPausable);
        assert!(!definition.repeatable);
        assert!(definition.lifecycle.can_disable());
        assert!(!definition.lifecycle.can_delete());
        assert!(definition.lifecycle.can_edit_execution_policy());
    }
}

fn context(file_cleanup: Arc<dyn FileCleanupPort>) -> TaskExecutionContext {
    TaskExecutionContext {
        http_client: Arc::new(UnexpectedHttpClient),
        system_cache: Arc::new(UnexpectedSystemCache),
        system_log_cleanup: Arc::new(UnexpectedSystemLogCleanup),
        file_cleanup,
    }
}

fn invocation(task_key: &str, task_params: serde_json::Value) -> TaskInvocation {
    TaskInvocation {
        execution_id: "execution-id".into(),
        job_id: "job-id".into(),
        task_key: task_key.into(),
        task_params,
        invoke_target: format!("{task_key}()"),
    }
}

#[derive(Default)]
struct RecordingFileCleanup {
    purge_calls: Mutex<Vec<(u64, u64)>>,
    upload_calls: Mutex<Vec<u64>>,
}

#[async_trait]
impl FileCleanupPort for RecordingFileCleanup {
    async fn purge_trash(&self, retention_days: u64, batch_size: u64) -> Result<FileTrashCleanupResult, TaskExecutionFailure> {
        self.purge_calls.lock().unwrap().push((retention_days, batch_size));
        Ok(FileTrashCleanupResult {
            purged_entries: 7,
            blocked_roots: 2,
            deleted_objects: 5,
            failed_objects: 0,
            retried_provider_cleanups: 3,
            failed_provider_cleanups: 0,
        })
    }

    async fn cleanup_upload_sessions(&self, batch_size: u64) -> Result<FileUploadSessionCleanupResult, TaskExecutionFailure> {
        self.upload_calls.lock().unwrap().push(batch_size);
        Ok(FileUploadSessionCleanupResult {
            expired_sessions: 4,
            reconciled_sessions: 2,
            retried_provider_cleanups: 1,
            failed_provider_cleanups: 0,
        })
    }
}

struct UnexpectedHttpClient;
struct UnexpectedSystemCache;
struct UnexpectedSystemLogCleanup;

#[async_trait]
impl HttpTaskClient for UnexpectedHttpClient {
    async fn send(&self, _: OutboundHttpRequest) -> Result<OutboundHttpResponse, OutboundHttpFailure> {
        panic!("file cleanup task invoked HTTP client")
    }
}

#[async_trait]
impl SystemCacheRefreshPort for UnexpectedSystemCache {
    async fn refresh_config_cache(&self) -> Result<(), TaskExecutionFailure> {
        panic!("file cleanup task invoked config cache refresh")
    }

    async fn refresh_dict_cache(&self) -> Result<(), TaskExecutionFailure> {
        panic!("file cleanup task invoked dictionary cache refresh")
    }
}

#[async_trait]
impl SystemLogCleanupPort for UnexpectedSystemLogCleanup {
    async fn cleanup_expired(&self, _: u64, _: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure> {
        panic!("file cleanup task invoked system log retention")
    }

    async fn cleanup_filtered(&self, _: SystemLogCleanupFilter, _: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure> {
        panic!("file cleanup task invoked system log cleanup")
    }
}
