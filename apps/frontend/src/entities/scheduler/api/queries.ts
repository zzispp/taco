import type { AxiosRequestConfig } from 'axios';
import type { PageResponse } from 'src/shared/api/types';
import type { QueryParams } from 'src/shared/api/pagination';
import type {
  SchedulerJob,
  ImportableTask,
  SchedulerJobLog,
  SchedulerJobLogQuery,
  SchedulerExecutionDetail,
} from '../model/types';

import useSWR from 'swr';
import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import { fetcher } from 'src/shared/api/http-client';
import { pageQuery } from 'src/shared/api/pagination';

import { schedulerEndpoints } from './endpoints';

type SchedulerPageRequest = {
  endpoint: string;
  page: number;
  pageSize: number;
  params: QueryParams;
  language: string;
};

type SchedulerPageKey = readonly [string, AxiosRequestConfig, string];
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

const schedulerSWRConfig = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useSchedulerJobs(page: number, pageSize: number, params: QueryParams = {}) {
  return useSchedulerPagedResource<SchedulerJob>({
    endpoint: schedulerEndpoints.jobs,
    page,
    pageSize,
    params,
    language: useSchedulerLanguage(),
  });
}

export function useSchedulerJobLogs(
  page: number,
  pageSize: number,
  params: SchedulerJobLogQuery = {}
) {
  return useSchedulerPagedResource<SchedulerJobLog>({
    endpoint: schedulerEndpoints.jobLogs,
    page,
    pageSize,
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

export function schedulerPageKey(request: SchedulerPageRequest): SchedulerPageKey {
  const config = { params: pageQuery(request.page, request.pageSize, request.params) };
  return [request.endpoint, config, request.language];
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

function useSchedulerPagedResource<T>(request: SchedulerPageRequest) {
  const { data, isLoading, error, isValidating } = useSWR<PageResponse<T>>(
    schedulerPageKey(request),
    fetchSchedulerPage,
    schedulerSWRConfig
  );
  return useMemo(
    () => ({
      data,
      items: data?.items ?? [],
      total: data?.total ?? 0,
      isLoading,
      error,
      isValidating,
    }),
    [data, error, isLoading, isValidating]
  );
}

function useSchedulerLanguage() {
  const { i18n } = useTranslation();
  return i18n.resolvedLanguage ?? i18n.language;
}

function fetchSchedulerPage<T>([endpoint, config]: SchedulerPageKey) {
  return fetcher<PageResponse<T>>([endpoint, config]);
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
