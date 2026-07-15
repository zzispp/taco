export const auditLogEndpoints = {
  operationLogs: '/api/system/operation-logs',
  operationLogsBatch: '/api/system/operation-logs/batch',
  operationLogsClean: '/api/system/operation-logs/clean',
  operationLogsExport: '/api/system/operation-logs/export',
  operationLog: (id: string) => `/api/system/operation-logs/${encodeURIComponent(id)}`,
  loginLogs: '/api/system/login-logs',
  loginLogsBatch: '/api/system/login-logs/batch',
  loginLogsClean: '/api/system/login-logs/clean',
  loginLogsExport: '/api/system/login-logs/export',
  loginLog: (id: string) => `/api/system/login-logs/${encodeURIComponent(id)}`,
  loginLogUnlock: (username: string) =>
    `/api/system/login-logs/${encodeURIComponent(username)}/unlock`,
} as const;
