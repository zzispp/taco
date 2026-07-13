import type {
  JobLogStatus,
  SchedulerJobLogQuery,
  SchedulerTriggerType,
} from 'src/entities/scheduler';

import {
  parseLocalDateTimeRange,
  LOCAL_DATE_TIME_FILTER_ERROR,
} from 'src/shared/lib/local-date-time-filter';

export const JOB_LOG_FILTER_ERROR = LOCAL_DATE_TIME_FILTER_ERROR;

export type JobLogFilterError = (typeof JOB_LOG_FILTER_ERROR)[keyof typeof JOB_LOG_FILTER_ERROR];

export type SchedulerJobLogFilterDraft = Readonly<{
  job_name: string;
  job_group: string;
  status: JobLogStatus | '';
  trigger_type: SchedulerTriggerType | '';
  begin_time: string;
  end_time: string;
}>;

export const DEFAULT_JOB_LOG_FILTER_DRAFT: SchedulerJobLogFilterDraft = Object.freeze({
  job_name: '',
  job_group: '',
  status: '',
  trigger_type: '',
  begin_time: '',
  end_time: '',
});

export const DEFAULT_JOB_LOG_QUERY: SchedulerJobLogQuery = Object.freeze({});

export type SchedulerJobLogFilterResult =
  | Readonly<{ ok: true; query: SchedulerJobLogQuery }>
  | Readonly<{ ok: false; error: JobLogFilterError }>;

export function isSchedulerJobLogQueryUsable(error: JobLogFilterError | null) {
  return error === null;
}

export function updateJobLogFilterState(
  currentQuery: SchedulerJobLogQuery,
  nextDraft: SchedulerJobLogFilterDraft
) {
  const result = toSchedulerJobLogQuery(nextDraft);
  return {
    draft: nextDraft,
    query: result.ok ? result.query : currentQuery,
    error: result.ok ? null : result.error,
    resetTable: result.ok,
  } as const;
}

export function toSchedulerJobLogQuery(
  draft: SchedulerJobLogFilterDraft
): SchedulerJobLogFilterResult {
  const dates = parseLocalDateTimeRange(draft.begin_time, draft.end_time);
  if (!dates.ok) return dates;
  const jobName = draft.job_name.trim();
  const jobGroup = draft.job_group.trim();
  return {
    ok: true,
    query: {
      ...(jobName && { job_name: jobName }),
      ...(jobGroup && { job_group: jobGroup }),
      ...(draft.status && { status: draft.status }),
      ...(draft.trigger_type && { trigger_type: draft.trigger_type }),
      ...(dates.begin && { begin_time: dates.begin.toISOString() }),
      ...(dates.end && { end_time: dates.end.toISOString() }),
    },
  };
}
