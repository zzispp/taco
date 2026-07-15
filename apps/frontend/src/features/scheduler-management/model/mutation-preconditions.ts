export const SCHEDULER_JOB_DELETE_TARGET_REQUIRED_ERROR =
  'A scheduler job delete target is required';
export const SCHEDULER_JOB_LOG_DELETE_TARGET_REQUIRED_ERROR =
  'A scheduler job log delete target is required';
export const SCHEDULER_JOB_LOG_EXPORT_FILTERS_REQUIRED_ERROR =
  'Valid scheduler job-log filters are required for export';

export function requireSchedulerJobDeleteTarget<T>(target: T | null): T {
  return requireTarget(target, SCHEDULER_JOB_DELETE_TARGET_REQUIRED_ERROR);
}

export function requireSchedulerJobLogDeleteTarget<T>(target: T | null): T {
  return requireTarget(target, SCHEDULER_JOB_LOG_DELETE_TARGET_REQUIRED_ERROR);
}

export function requireUsableSchedulerJobLogFilters(filtersValid: boolean): void {
  if (!filtersValid) {
    throw new Error(SCHEDULER_JOB_LOG_EXPORT_FILTERS_REQUIRED_ERROR);
  }
}

function requireTarget<T>(target: T | null, message: string): T {
  if (target === null) {
    throw new Error(message);
  }
  return target;
}
