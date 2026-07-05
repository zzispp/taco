export const menuEndpoints = {
  menus: '/api/system/menus',
  menuTree: '/api/system/menus/tree',
  treeSelect: '/api/system/menus/tree-select',
  roleTreeSelect: (id: string) => `/api/system/menus/role-tree-select/${id}`,
  menu: (id: string) => `/api/system/menus/${id}`,
  sort: (id: string) => `/api/system/menus/${id}/sort`,
  sortBatch: '/api/system/menus/sort',
} as const;
