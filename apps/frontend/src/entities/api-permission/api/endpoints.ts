export const apiPermissionEndpoints = {
  apis: '/api/rbac/apis',
  api: (id: string) => `/api/rbac/apis/${id}`,
} as const;
