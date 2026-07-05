import type { TreeSelectNode } from 'src/entities/system';

export type Role = {
  role_id: string;
  role_name: string;
  role_key: string;
  role_sort: number;
  data_scope: string;
  menu_check_strictly: boolean;
  dept_check_strictly: boolean;
  status: string;
  system: boolean;
  remark: string | null;
  create_time: string;
};

export type RoleSummary = {
  role_id: string;
  role_name: string;
  role_key: string;
};

export type RoleOption = RoleSummary & {
  role_sort: number;
  status: string;
};

export type RoleInput = {
  role_name: string;
  role_key: string;
  role_sort: number;
  data_scope: string;
  menu_check_strictly: boolean;
  dept_check_strictly: boolean;
  status: string;
  remark: string | null;
};

export type RoleMenuBinding = {
  menu_ids: string[];
};

export type RoleDeptBinding = {
  dept_ids: string[];
};

export type RoleMenuTreeSelect = {
  menus: TreeSelectNode[];
  checked_keys: string[];
};

export type RoleDeptTreeSelect = {
  depts: TreeSelectNode[];
  checked_keys: string[];
};

export type RoleDataScopeInput = {
  data_scope: string;
  dept_check_strictly: boolean;
  dept_ids: string[];
};

export type RoleUser = {
  user_id: string;
  username: string;
  nick_name: string;
  dept_id: string | null;
  dept_name: string | null;
  phonenumber: string | null;
  email: string;
  status: string;
};

export type RoleUserBinding = {
  user_ids: string[];
};
