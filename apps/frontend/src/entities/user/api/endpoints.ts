export const userEndpoints = {
  users: '/api/system/users',
  usersBatch: '/api/system/users/batch',
  user: (id: string) => `/api/system/users/${id}`,
  status: (id: string) => `/api/system/users/${id}/status`,
  password: (id: string) => `/api/system/users/${id}/password`,
  roles: (id: string) => `/api/system/users/${id}/roles`,
  deptTree: '/api/system/users/dept-tree',
  formOptions: '/api/system/users/form-options',
} as const;
