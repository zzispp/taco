import { it, expect, describe } from 'vitest';

import { schedulerEndpoints } from './endpoints';
import { JOB_LOG_STATUS } from '../model/constants';
import {
  schedulerJobKey,
  schedulerCursorKey,
  schedulerJobLogDetailKey,
  importableSchedulerTasksKey,
} from './queries';

describe('importable scheduler task query key', () => {
  it('does not request data without import permission', () => {
    expect(importableSchedulerTasksKey(false, 'en')).toBeNull();
  });

  it('includes language when import permission is present', () => {
    expect(importableSchedulerTasksKey(true, 'tw')).toEqual([
      schedulerEndpoints.importableJobs,
      'tw',
    ]);
  });
});

describe('localized scheduler cursor query key', () => {
  const request = {
    endpoint: schedulerEndpoints.jobLogs,
    request: { limit: 20, cursor: 'next-log' },
    params: { status: JOB_LOG_STATUS.FAILED },
  };

  it('uses language as cache identity without adding a locale query parameter', () => {
    const key = schedulerCursorKey({ ...request, language: 'tw' });

    expect(key[1]).toEqual({
      params: { limit: 20, cursor: 'next-log', status: JOB_LOG_STATUS.FAILED },
    });
    expect(key[1].params).not.toHaveProperty('language');
    expect(key[2]).toBe('tw');
  });

  it('creates distinct cache keys for different resolved languages', () => {
    const cn = schedulerCursorKey({ ...request, language: 'cn' });
    const en = schedulerCursorKey({ ...request, language: 'en' });

    expect(cn).not.toEqual(en);
  });
});

describe('scheduler execution detail query key', () => {
  const request = {
    executionId: 'execution-1',
    canQuery: true,
    canDetail: true,
    language: 'en',
  };

  it.each([
    [{ ...request, executionId: null }, null],
    [{ ...request, canQuery: false }, null],
    [{ ...request, canDetail: false }, null],
  ] as const)('does not create a key without selection and both permissions', (input, expected) => {
    expect(schedulerJobLogDetailKey(input)).toBe(expected);
  });

  it('includes the execution and current language in the cache identity', () => {
    expect(schedulerJobLogDetailKey({ ...request, language: 'tw' })).toEqual([
      schedulerEndpoints.jobLogDetail('execution-1'),
      'tw',
    ]);
  });
});

describe('scheduler job detail query key', () => {
  const request = {
    jobId: 'job-1',
    canQuery: true,
    language: 'en',
  };

  it.each([
    [{ ...request, jobId: null }, null],
    [{ ...request, canQuery: false }, null],
  ] as const)('does not create a key without selection and query permission', (input, expected) => {
    expect(schedulerJobKey(input)).toBe(expected);
  });

  it('includes the job and current language in the cache identity', () => {
    expect(schedulerJobKey({ ...request, language: 'tw' })).toEqual([
      schedulerEndpoints.job('job-1'),
      'tw',
    ]);
  });
});
