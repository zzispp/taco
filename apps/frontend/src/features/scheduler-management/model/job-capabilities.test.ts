import { it, expect, describe } from 'vitest';

import { REGISTRY_STATUS } from 'src/entities/scheduler';

import { schedulerJobCapabilities, selectableSchedulerJobIds } from './job-capabilities';

describe('scheduler job capabilities', () => {
  const administrable = {
    can_disable: true,
    can_delete: true,
    can_edit_execution_policy: true,
  };

  it.each([
    [REGISTRY_STATUS.OK, { editable: true, runnable: true }],
    [REGISTRY_STATUS.INVALID_PARAMS, { editable: true, runnable: false }],
    [REGISTRY_STATUS.MISSING, { editable: false, runnable: false }],
    [REGISTRY_STATUS.REPEATABLE_MISMATCH, { editable: false, runnable: false }],
  ] as const)('projects registry status %s to backend-aligned capabilities', (status, expected) => {
    expect(schedulerJobCapabilities(status, administrable)).toEqual({
      ...expected,
      canDisable: true,
      canDelete: true,
      canEditExecutionPolicy: true,
    });
  });

  it('keeps required-task restrictions from the backend', () => {
    expect(
      schedulerJobCapabilities(REGISTRY_STATUS.OK, {
        can_disable: false,
        can_delete: false,
        can_edit_execution_policy: false,
      })
    ).toEqual({
      editable: true,
      runnable: true,
      canDisable: false,
      canDelete: false,
      canEditExecutionPolicy: false,
    });
  });

  it('selects only deletable jobs for a bulk delete request', () => {
    const jobs = [job('required', false), job('deletable', true)];

    expect(selectableSchedulerJobIds(jobs, true)).toEqual(['deletable']);
    expect(selectableSchedulerJobIds(jobs, false)).toEqual([]);
  });
});

function job(job_id: string, can_delete: boolean) {
  return {
    job_id,
    registry_status: REGISTRY_STATUS.OK,
    capabilities: {
      can_disable: can_delete,
      can_delete,
      can_edit_execution_policy: can_delete,
    },
  } as Parameters<typeof selectableSchedulerJobIds>[0][number];
}
