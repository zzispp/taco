import type { DeptInput } from 'src/entities/system';

export const DEFAULT_INPUT: DeptInput = {
  parent_id: '0',
  dept_name: '',
  order_num: 0,
  leader: '',
  phone: '',
  email: '',
  status: '0',
};

export const DEFAULT_FILTERS = { dept_name: '', status: '' };

export type DeptFiltersValue = typeof DEFAULT_FILTERS;
