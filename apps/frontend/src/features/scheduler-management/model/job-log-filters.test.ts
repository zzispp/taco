import { it, expect, describe } from 'vitest';

import { JOB_LOG_STATUS, SCHEDULER_TRIGGER_TYPE } from 'src/entities/scheduler';

import {
  JOB_LOG_FILTER_ERROR,
  toSchedulerJobLogQuery,
  updateJobLogFilterState,
  isSchedulerJobLogQueryUsable,
  DEFAULT_JOB_LOG_FILTER_DRAFT,
} from './job-log-filters';

describe('scheduler job log query conversion', () => {
  it('omits empty filters', () => {
    expect(toSchedulerJobLogQuery(DEFAULT_JOB_LOG_FILTER_DRAFT)).toEqual({
      ok: true,
      query: {},
    });
  });

  it('trims text and converts local timestamps to UTC RFC3339', () => {
    const result = toSchedulerJobLogQuery({
      job_name: '  HTTP probe  ',
      job_group: ' SYSTEM ',
      status: JOB_LOG_STATUS.FAILED,
      trigger_type: SCHEDULER_TRIGGER_TYPE.MANUAL,
      begin_time: '2026-07-10T18:12',
      end_time: '2026-07-10T19:12',
    });

    expect(result).toEqual({
      ok: true,
      query: {
        job_name: 'HTTP probe',
        job_group: 'SYSTEM',
        status: JOB_LOG_STATUS.FAILED,
        trigger_type: SCHEDULER_TRIGGER_TYPE.MANUAL,
        begin_time: new Date(2026, 6, 10, 18, 12).toISOString(),
        end_time: new Date(2026, 6, 10, 19, 12).toISOString(),
      },
    });
  });
});

describe('scheduler job log date range validation', () => {
  it.each(['not-a-date', '2026-02-30T18:12', '2026-07-10T24:00'])(
    'rejects invalid local timestamp %s',
    (beginTime) => {
      expect(
        toSchedulerJobLogQuery({
          ...DEFAULT_JOB_LOG_FILTER_DRAFT,
          begin_time: beginTime,
        })
      ).toEqual({ ok: false, error: JOB_LOG_FILTER_ERROR.INVALID_DATE_TIME });
    }
  );

  it('rejects a reversed inclusive range', () => {
    expect(
      toSchedulerJobLogQuery({
        ...DEFAULT_JOB_LOG_FILTER_DRAFT,
        begin_time: '2026-07-10T19:12',
        end_time: '2026-07-10T18:12',
      })
    ).toEqual({ ok: false, error: JOB_LOG_FILTER_ERROR.INVALID_RANGE });
  });

  it('accepts equal inclusive range endpoints', () => {
    const value = '2026-07-10T18:12';
    const timestamp = new Date(2026, 6, 10, 18, 12).toISOString();

    expect(
      toSchedulerJobLogQuery({
        ...DEFAULT_JOB_LOG_FILTER_DRAFT,
        begin_time: value,
        end_time: value,
      })
    ).toEqual({
      ok: true,
      query: { begin_time: timestamp, end_time: timestamp },
    });
  });
});

describe('scheduler job log filter state transition', () => {
  it('resets pagination and selection when applying a valid filter', () => {
    expect(
      updateJobLogFilterState(
        { job_group: 'OLD' },
        { ...DEFAULT_JOB_LOG_FILTER_DRAFT, job_group: 'NEW' }
      )
    ).toEqual({
      draft: { ...DEFAULT_JOB_LOG_FILTER_DRAFT, job_group: 'NEW' },
      query: { job_group: 'NEW' },
      error: null,
      resetTable: true,
    });
  });

  it('retains the last valid query while exposing an invalid draft', () => {
    const previousQuery = { job_group: 'SYSTEM' };
    const invalidDraft = {
      ...DEFAULT_JOB_LOG_FILTER_DRAFT,
      begin_time: '2026-07-10T19:12',
      end_time: '2026-07-10T18:12',
    };

    expect(updateJobLogFilterState(previousQuery, invalidDraft)).toEqual({
      draft: invalidDraft,
      query: previousQuery,
      error: JOB_LOG_FILTER_ERROR.INVALID_RANGE,
      resetTable: false,
    });
  });

  it('blocks query-dependent actions while the visible draft is invalid', () => {
    expect(isSchedulerJobLogQueryUsable(null)).toBe(true);
    expect(isSchedulerJobLogQueryUsable(JOB_LOG_FILTER_ERROR.INVALID_RANGE)).toBe(false);
  });
});
