export const roleEndpoints = {
  roles: '/api/rbac/roles',
  role: (code: string) => `/api/rbac/roles/${code}`,
  roleApis: (code: string) => `/api/rbac/roles/${code}/apis`,
  roleMenus: (code: string) => `/api/rbac/roles/${code}/menus`,
} as const;
