import type { RoleInput } from 'src/entities/role';

export const DEFAULT_FORM: RoleInput = {
  role_name: '',
  role_key: '',
  role_sort: 0,
  data_scope: '5',
  menu_check_strictly: true,
  dept_check_strictly: true,
  status: '0',
  remark: '',
};

export const DEFAULT_FILTERS = {
  role_name: '',
  role_key: '',
  status: '',
  begin_time: '',
  end_time: '',
};

export type RoleFiltersValue = typeof DEFAULT_FILTERS;
