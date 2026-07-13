import type { SchedulerJob } from 'src/entities/scheduler';

export const EMPTY_JOB_DETAIL_VALUE = '\u2014';

export const JOB_DETAIL_TAB = {
  CONFIGURATION: 'configuration',
  SCHEDULE: 'schedule',
  METHOD: 'method',
  METADATA: 'metadata',
} as const;

export type JobDetailTab = (typeof JOB_DETAIL_TAB)[keyof typeof JOB_DETAIL_TAB];

export function isCurrentJobDetail<T extends Pick<SchedulerJob, 'job_id'>>(
  target: Pick<SchedulerJob, 'job_id'> | null,
  detail: T | undefined
): detail is T {
  return Boolean(target && detail && target.job_id === detail.job_id);
}

export function formatTaskParameters(taskParams: Readonly<Record<string, unknown>>) {
  return JSON.stringify(taskParams, null, 2);
}

export function jobDetailDisplayValue(value: string | null | undefined) {
  return value?.trim() ? value : EMPTY_JOB_DETAIL_VALUE;
}

export function formatRuntimeErrorDetail(codeLabel: string, message: string) {
  return message === codeLabel ? codeLabel : `${codeLabel}: ${message}`;
}
