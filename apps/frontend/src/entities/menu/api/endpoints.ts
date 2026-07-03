export const menuEndpoints = {
  menuSections: '/api/rbac/menu-sections',
  menuSection: (id: string) => `/api/rbac/menu-sections/${id}`,
  menuItems: '/api/rbac/menu-items',
  menuItem: (id: string) => `/api/rbac/menu-items/${id}`,
} as const;
