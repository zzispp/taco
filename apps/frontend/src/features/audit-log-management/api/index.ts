import type { LoginLogListQuery, OperationLogListQuery } from 'src/entities/audit-log';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { downloadBlobResponse } from 'src/shared/api/download';
import { compactParams, isEndpointKey } from 'src/shared/api/pagination';

import { auditLogEndpoints } from 'src/entities/audit-log';

export async function deleteOperationLog(id: string) {
  await axios.delete(auditLogEndpoints.operationLog(id));
  await refreshAuditLists(auditLogEndpoints.operationLogs);
}

export async function deleteOperationLogs(ids: readonly string[]) {
  await axios.delete(auditLogEndpoints.operationLogsBatch, { data: { ids } });
  await refreshAuditLists(auditLogEndpoints.operationLogs);
}

export async function cleanOperationLogs() {
  await axios.delete(auditLogEndpoints.operationLogsClean);
  await refreshAuditLists(auditLogEndpoints.operationLogs);
}

export async function exportOperationLogs(query: OperationLogListQuery) {
  const response = await axios.post<Blob>(auditLogEndpoints.operationLogsExport, null, {
    params: compactParams(query),
    responseType: 'blob',
  });
  downloadBlobResponse(response, 'operation_logs.xlsx');
  await refreshAuditLists(auditLogEndpoints.operationLogs);
}

export async function deleteLoginLog(id: string) {
  await axios.delete(auditLogEndpoints.loginLog(id));
  await refreshAuditLists(auditLogEndpoints.loginLogs, auditLogEndpoints.operationLogs);
}

export async function deleteLoginLogs(ids: readonly string[]) {
  await axios.delete(auditLogEndpoints.loginLogsBatch, { data: { ids } });
  await refreshAuditLists(auditLogEndpoints.loginLogs, auditLogEndpoints.operationLogs);
}

export async function cleanLoginLogs() {
  await axios.delete(auditLogEndpoints.loginLogsClean);
  await refreshAuditLists(auditLogEndpoints.loginLogs, auditLogEndpoints.operationLogs);
}

export async function exportLoginLogs(query: LoginLogListQuery) {
  const response = await axios.post<Blob>(auditLogEndpoints.loginLogsExport, null, {
    params: compactParams(query),
    responseType: 'blob',
  });
  downloadBlobResponse(response, 'login_logs.xlsx');
  await refreshAuditLists(auditLogEndpoints.operationLogs);
}

export async function unlockLoginAccount(username: string) {
  await axios.put(auditLogEndpoints.loginLogUnlock(username));
  await refreshAuditLists(auditLogEndpoints.operationLogs);
}

async function refreshAuditLists(...endpoints: readonly string[]) {
  await Promise.all(endpoints.map((endpoint) => mutate((key) => isEndpointKey(key, endpoint))));
}
