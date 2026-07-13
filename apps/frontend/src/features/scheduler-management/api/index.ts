import type {
  JobStatus,
  SchedulerJob,
  ImportJobInput,
  RunJobResponse,
  ReplaceJobInput,
  SchedulerJobLogQuery,
} from 'src/entities/scheduler';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { downloadBlobResponse } from 'src/shared/api/download';
import { requestData, compactParams, isEndpointKey } from 'src/shared/api/pagination';

import { schedulerEndpoints } from 'src/entities/scheduler';

export type SchedulerCacheCapabilities = Readonly<{
  canRefreshImportableTasks: boolean;
}>;

export async function importJob(payload: ImportJobInput, capabilities: SchedulerCacheCapabilities) {
  const item = await requestData<SchedulerJob>(axios.post(schedulerEndpoints.importJob, payload));
  await refreshJobs(capabilities);
  return item;
}

export async function updateJob(
  id: string,
  payload: ReplaceJobInput,
  capabilities: SchedulerCacheCapabilities
) {
  const item = await requestData<SchedulerJob>(axios.put(schedulerEndpoints.job(id), payload));
  await refreshJobs(capabilities);
  await clearJobDetailCache([id]);
  return item;
}

export async function deleteJob(id: string, capabilities: SchedulerCacheCapabilities) {
  await axios.delete(schedulerEndpoints.job(id));
  await refreshJobs(capabilities);
  await clearJobDetailCache([id]);
}

export async function deleteJobs(ids: string[], capabilities: SchedulerCacheCapabilities) {
  await axios.delete(schedulerEndpoints.jobsBatch, { data: { ids } });
  await refreshJobs(capabilities);
  await clearJobDetailCache(ids);
}

export async function updateJobStatus(
  id: string,
  status: JobStatus,
  capabilities: SchedulerCacheCapabilities
) {
  const item = await requestData<SchedulerJob>(
    axios.put(schedulerEndpoints.jobStatus(id), { status })
  );
  await refreshJobs(capabilities);
  await clearJobDetailCache([id]);
  return item;
}

export async function runJob(id: string) {
  return requestData<RunJobResponse>(axios.post(schedulerEndpoints.jobRun(id)));
}

export async function exportJobs(filters: Record<string, string> = {}) {
  const response = await axios.post<Blob>(schedulerEndpoints.jobsExport, null, {
    params: compactParams(filters),
    responseType: 'blob',
  });
  downloadBlobResponse(response, 'jobs.xlsx');
}

export async function deleteJobLog(id: string) {
  await axios.delete(schedulerEndpoints.jobLog(id));
  await refreshJobLogs();
}

export async function deleteJobLogs(ids: string[]) {
  await axios.delete(schedulerEndpoints.jobLogsBatch, { data: { ids } });
  await refreshJobLogs();
}

export async function clearJobLogs() {
  await axios.delete(schedulerEndpoints.jobLogsClean);
  await refreshJobLogs();
}

export async function exportJobLogs(filters: SchedulerJobLogQuery = {}) {
  const response = await axios.post<Blob>(schedulerEndpoints.jobLogsExport, null, {
    params: compactParams({ ...filters }),
    responseType: 'blob',
  });
  downloadBlobResponse(response, 'job_logs.xlsx');
}

export async function cronNextTimes(expression: string) {
  return requestData<{ times: string[] }>(
    axios.post(schedulerEndpoints.cronNextTimes, { expression })
  );
}

export function schedulerJobRefreshEndpoints(capabilities: SchedulerCacheCapabilities) {
  return capabilities.canRefreshImportableTasks
    ? [schedulerEndpoints.jobs, schedulerEndpoints.importableJobs]
    : [schedulerEndpoints.jobs];
}

export function schedulerJobDetailCacheMatcher(ids: readonly string[]) {
  const endpoints = new Set(ids.map(schedulerEndpoints.job));
  return (key: unknown) => [...endpoints].some((endpoint) => isEndpointKey(key, endpoint));
}

async function refreshJobs(capabilities: SchedulerCacheCapabilities) {
  for (const endpoint of schedulerJobRefreshEndpoints(capabilities)) {
    await mutate((key) => isEndpointKey(key, endpoint));
  }
}

async function clearJobDetailCache(ids: string[]) {
  await mutate(schedulerJobDetailCacheMatcher(ids), undefined, { revalidate: false });
}

async function refreshJobLogs() {
  await mutate((key) => isEndpointKey(key, schedulerEndpoints.jobLogs));
}
