import type {
  SystemLogFilterQuery,
  SystemLogCleanupCount,
  SystemLogCleanupAccepted,
} from 'src/entities/system-log';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { downloadBlobResponse } from 'src/shared/api/download';
import { compactParams, isEndpointKey } from 'src/shared/api/pagination';

import { systemLogEndpoints } from 'src/entities/system-log';

export async function deleteSystemLog(id: string) {
  await axios.delete(systemLogEndpoints.detail(id));
  await refreshSystemLogs();
}

export async function deleteSystemLogs(ids: readonly string[]) {
  await axios.delete(systemLogEndpoints.batch, { data: { ids } });
  await refreshSystemLogs();
}

export async function countSystemLogsForCleanup(query: SystemLogFilterQuery) {
  const response = await axios.get<SystemLogCleanupCount>(systemLogEndpoints.cleanCount, {
    params: compactParams(query),
  });
  return response.data;
}

export async function cleanSystemLogs(query: SystemLogFilterQuery) {
  const response = await axios.delete<SystemLogCleanupAccepted>(systemLogEndpoints.clean, {
    params: compactParams(query),
  });
  return response.data;
}

export async function exportSystemLogs(query: SystemLogFilterQuery) {
  const response = await axios.post<Blob>(systemLogEndpoints.export, null, {
    params: compactParams(query),
    responseType: 'blob',
  });
  downloadBlobResponse(response, 'system_logs.xlsx');
}

export async function refreshSystemLogs() {
  await mutate((key) => isEndpointKey(key, systemLogEndpoints.logs));
}
