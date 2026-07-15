import { it, expect, describe } from 'vitest';

import {
  requireSchedulerJobDeleteTarget,
  requireSchedulerJobLogDeleteTarget,
  requireUsableSchedulerJobLogFilters,
  SCHEDULER_JOB_DELETE_TARGET_REQUIRED_ERROR,
  SCHEDULER_JOB_LOG_DELETE_TARGET_REQUIRED_ERROR,
  SCHEDULER_JOB_LOG_EXPORT_FILTERS_REQUIRED_ERROR,
} from './mutation-preconditions';

describe('scheduler mutation preconditions', () => {
  it('rejects a scheduler job delete without a selected target', () => {
    expect(() => requireSchedulerJobDeleteTarget(null)).toThrowError(
      SCHEDULER_JOB_DELETE_TARGET_REQUIRED_ERROR
    );
  });

  it('preserves the selected scheduler job delete target', () => {
    const target = { job_id: 'job-1' };

    expect(requireSchedulerJobDeleteTarget(target)).toBe(target);
  });

  it('does not confuse a non-null target with a missing target', () => {
    expect(requireSchedulerJobDeleteTarget(0)).toBe(0);
  });

  it('rejects a scheduler job-log delete without a selected target', () => {
    expect(() => requireSchedulerJobLogDeleteTarget(null)).toThrowError(
      SCHEDULER_JOB_LOG_DELETE_TARGET_REQUIRED_ERROR
    );
  });

  it('rejects export while the visible job-log filter draft is invalid', () => {
    expect(() => requireUsableSchedulerJobLogFilters(false)).toThrowError(
      SCHEDULER_JOB_LOG_EXPORT_FILTERS_REQUIRED_ERROR
    );
  });

  it('allows export after job-log filters pass validation', () => {
    expect(() => requireUsableSchedulerJobLogFilters(true)).not.toThrow();
  });
});
