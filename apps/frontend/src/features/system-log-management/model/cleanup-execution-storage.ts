const CLEANUP_EXECUTION_STORAGE_KEY = 'system-log-cleanup-execution-id';

export function loadCleanupExecutionId() {
  if (typeof window === 'undefined') return null;
  return window.sessionStorage.getItem(CLEANUP_EXECUTION_STORAGE_KEY);
}

export function storeCleanupExecutionId(executionId: string | null) {
  if (typeof window === 'undefined') return;
  if (executionId) {
    window.sessionStorage.setItem(CLEANUP_EXECUTION_STORAGE_KEY, executionId);
    return;
  }
  window.sessionStorage.removeItem(CLEANUP_EXECUTION_STORAGE_KEY);
}
