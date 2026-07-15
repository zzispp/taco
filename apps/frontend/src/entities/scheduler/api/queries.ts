import type { QueryParams, CursorPageRequest } from 'src/shared/api/pagination';
import type {
  SchedulerJob,
  ImportableTask,
  SchedulerJobLog,
  SchedulerJobLogQuery,
  SchedulerExecutionDetail,
} from '../model/types';

import useSWR from 'swr';
import { useTranslation } from 'react-i18next';

import { fetcher } from 'src/shared/api/http-client';
import { cursorResourceKey, useCursorResource } from 'src/shared/api/use-cursor-resource';

import { schedulerEndpoints } from './endpoints';

type SchedulerCursorRequest = {
  endpoint: string;
  request: CursorPageRequest;
  params: QueryParams;
  language: string;
};

type SchedulerJobKey = readonly [string, string];
type SchedulerJobLogDetailKey = readonly [string, string];

type SchedulerJobAccess = Readonly<{
  jobId: string | null;
  canQuery: boolean;
}>;

type SchedulerJobKeyRequest = SchedulerJobAccess & Readonly<{ language: string }>;

type SchedulerJobLogDetailAccess = Readonly<{
  executionId: string | null;
  canQuery: boolean;
  canDetail: boolean;
}>;

type SchedulerJobLogDetailKeyRequest = SchedulerJobLogDetailAccess & Readonly<{ language: string }>;

export function useSchedulerJobs(request: CursorPageRequest, params: QueryParams = {}) {
  return useSchedulerCursorResource<SchedulerJob>({
    endpoint: schedulerEndpoints.jobs,
    request,
    params,
    language: useSchedulerLanguage(),
  });
}

export function useSchedulerJobLogs(request: CursorPageRequest, params: SchedulerJobLogQuery = {}) {
  return useSchedulerCursorResource<SchedulerJobLog>({
    endpoint: schedulerEndpoints.jobLogs,
    request,
    params,
    language: useSchedulerLanguage(),
  });
}

export function useSchedulerJob(access: SchedulerJobAccess) {
  const key = schedulerJobKey({ ...access, language: useSchedulerLanguage() });
  return useSWR<SchedulerJob>(key, fetchSchedulerJob, {
    revalidateOnFocus: false,
  });
}

export function useImportableSchedulerTasks(enabled: boolean) {
  const language = useSchedulerLanguage();
  const key = importableSchedulerTasksKey(enabled, language);
  return useSWR<ImportableTask[]>(key, fetchImportableTasks, {
    revalidateOnFocus: false,
  });
}

export function useSchedulerJobLogDetail(access: SchedulerJobLogDetailAccess) {
  const key = schedulerJobLogDetailKey({ ...access, language: useSchedulerLanguage() });
  return useSWR<SchedulerExecutionDetail>(key, fetchSchedulerJobLogDetail, {
    revalidateOnFocus: false,
  });
}

export function schedulerCursorKey(request: SchedulerCursorRequest) {
  const key = cursorResourceKey({
    endpoint: request.endpoint,
    request: request.request,
    params: request.params,
    context: request.language,
  });
  if (!key) throw new Error('Scheduler cursor endpoint is required');
  return key;
}

export function importableSchedulerTasksKey(enabled: boolean, language: string) {
  return enabled ? ([schedulerEndpoints.importableJobs, language] as const) : null;
}

export function schedulerJobKey(request: SchedulerJobKeyRequest) {
  if (!request.jobId || !request.canQuery) return null;
  return [schedulerEndpoints.job(request.jobId), request.language] as const;
}

export function schedulerJobLogDetailKey(request: SchedulerJobLogDetailKeyRequest) {
  if (!request.executionId || !request.canQuery || !request.canDetail) return null;
  return [schedulerEndpoints.jobLogDetail(request.executionId), request.language] as const;
}

function useSchedulerCursorResource<T>(request: SchedulerCursorRequest) {
  return useCursorResource<T>({
    endpoint: request.endpoint,
    request: request.request,
    params: request.params,
    context: request.language,
  });
}

function useSchedulerLanguage() {
  const { i18n } = useTranslation();
  return i18n.resolvedLanguage ?? i18n.language;
}

function fetchImportableTasks([endpoint]: readonly [string, string]) {
  return fetcher<ImportableTask[]>(endpoint);
}

function fetchSchedulerJob([endpoint]: SchedulerJobKey) {
  return fetcher<SchedulerJob>(endpoint);
}

function fetchSchedulerJobLogDetail([endpoint]: SchedulerJobLogDetailKey) {
  return fetcher<SchedulerExecutionDetail>(endpoint);
}
