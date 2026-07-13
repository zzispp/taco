import { it, expect, describe } from 'vitest';

import {
  JOB_LOG_STATUS,
  REGISTRY_STATUS,
  RUNTIME_ERROR_CODE,
  SCHEDULER_PERMISSION,
  SCHEDULER_TRIGGER_TYPE,
  jobLogStatusTranslationKeys,
  runtimeErrorTranslationKeys,
  registryStatusTranslationKeys,
  schedulerTriggerTranslationKeys,
} from './constants';

describe('scheduler status projections', () => {
  it('covers every registry and runtime error status with an explicit translation key', () => {
    expect(Object.keys(registryStatusTranslationKeys).sort()).toEqual(
      Object.values(REGISTRY_STATUS).sort()
    );
    expect(Object.keys(runtimeErrorTranslationKeys).sort()).toEqual(
      Object.values(RUNTIME_ERROR_CODE).sort()
    );
  });

  it('covers every execution outcome with an explicit translation key', () => {
    expect(Object.keys(jobLogStatusTranslationKeys).sort()).toEqual(
      Object.values(JOB_LOG_STATUS).sort()
    );
  });

  it('covers every scheduler trigger with an explicit translation key', () => {
    expect(Object.keys(schedulerTriggerTranslationKeys).sort()).toEqual(
      Object.values(SCHEDULER_TRIGGER_TYPE).sort()
    );
  });
});

describe('scheduler permissions', () => {
  it('owns every permission used by scheduler UI capabilities', () => {
    expect(Object.values(SCHEDULER_PERMISSION).sort()).toEqual([
      'system:job:changeStatus',
      'system:job:edit',
      'system:job:export',
      'system:job:import',
      'system:job:log:detail',
      'system:job:log:export',
      'system:job:log:query',
      'system:job:log:remove',
      'system:job:query',
      'system:job:remove',
      'system:job:run',
    ]);
  });
});
