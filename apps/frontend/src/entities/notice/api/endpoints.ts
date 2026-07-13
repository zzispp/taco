export const noticeEndpoints = {
  notices: '/api/system/notices',
  noticesBatch: '/api/system/notices/batch',
  top: '/api/system/notices/top',
  readAll: '/api/system/notices/read-all',
  notice: (id: string) => `/api/system/notices/${id}`,
  readers: (id: string) => `/api/system/notices/${id}/readers`,
  read: (id: string) => `/api/system/notices/${id}/read`,
} as const;
