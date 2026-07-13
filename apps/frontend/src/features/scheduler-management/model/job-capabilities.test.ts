import { it, expect, describe } from 'vitest';

import { REGISTRY_STATUS } from 'src/entities/scheduler';

import { schedulerJobCapabilities } from './job-capabilities';

describe('scheduler job capabilities', () => {
  it.each([
    [REGISTRY_STATUS.OK, { editable: true, runnable: true }],
    [REGISTRY_STATUS.INVALID_PARAMS, { editable: true, runnable: false }],
    [REGISTRY_STATUS.MISSING, { editable: false, runnable: false }],
    [REGISTRY_STATUS.REPEATABLE_MISMATCH, { editable: false, runnable: false }],
  ] as const)('projects registry status %s to backend-aligned capabilities', (status, expected) => {
    expect(schedulerJobCapabilities(status)).toEqual(expected);
  });
});
