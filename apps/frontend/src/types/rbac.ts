// ----------------------------------------------------------------------

export type PageResponse<T> = {
  items: T[];
  total: number;
  page: number;
  page_size: number;
};

export type Role = {
  code: string;
  name: string;
  description: string;
  enabled: boolean;
  system: boolean;
  sort_order: number;
};

export type RoleInput = {
  code: string;
  name: string;
  description: string;
  enabled: boolean;
  sort_order: number;
};

export type ApiPermission = {
  id: string;
  code: string;
  method: string;
  path_pattern: string;
  name: string;
  group: string;
  enabled: boolean;
  system: boolean;
};

export type ApiPermissionInput = {
  code: string;
  method: string;
  path_pattern: string;
  name: string;
  group: string;
  enabled: boolean;
};

export type MenuSection = {
  id: string;
  code: string;
  subheader: string;
  sort_order: number;
  enabled: boolean;
};

export type MenuSectionInput = {
  code: string;
  subheader: string;
  sort_order: number;
  enabled: boolean;
};

export type MenuItem = {
  id: string;
  section_id: string;
  parent_id: string | null;
  code: string;
  title: string;
  path: string;
  icon: string | null;
  caption: string | null;
  deep_match: boolean;
  sort_order: number;
  enabled: boolean;
};

export type MenuItemInput = {
  section_id: string;
  parent_id: string | null;
  code: string;
  title: string;
  path: string;
  icon: string | null;
  caption: string | null;
  deep_match: boolean;
  sort_order: number;
  enabled: boolean;
};

export type RoleApiBinding = {
  api_permission_ids: string[];
};

export type RoleMenuBinding = {
  menu_item_ids: string[];
};

export type NavResponse = {
  nav_items: BackendNavSection[];
};

export type BackendNavSection = {
  code: string;
  subheader: string;
  items: BackendNavItem[];
};

export type BackendNavItem = {
  code: string;
  title: string;
  path: string;
  icon: string | null;
  caption: string | null;
  deep_match: boolean;
  children: BackendNavItem[];
};

export type SystemUser = {
  id: string;
  username: string;
  email: string;
  role: string;
  is_active: boolean;
  auth_source: string;
  email_verified: boolean;
  system: boolean;
};

export type UserInput = {
  username: string;
  password: string;
  email: string;
  role: string;
  is_active: boolean;
};
