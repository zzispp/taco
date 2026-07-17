export const systemLogEndpoints = {
  logs: '/api/system/system-logs',
  batch: '/api/system/system-logs/batch',
  clean: '/api/system/system-logs/clean',
  cleanCount: '/api/system/system-logs/clean/count',
  cleanExecution: (id: string) =>
    `/api/system/system-logs/clean/executions/${encodeURIComponent(id)}`,
  export: '/api/system/system-logs/export',
  detail: (id: string) => `/api/system/system-logs/${encodeURIComponent(id)}`,
} as const;
