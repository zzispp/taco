import type { UserInput } from 'src/entities/user';

export const DEFAULT_FORM: UserInput = {
  username: '',
  password: '',
  nick_name: '',
  dept_id: null,
  email: '',
  phonenumber: '',
  sex: '2',
  status: '0',
  remark: '',
  role_ids: [],
  post_ids: [],
};

export const DEFAULT_FILTERS = {
  username: '',
  phonenumber: '',
  status: '',
  dept_id: '',
  begin_time: '',
  end_time: '',
};

export const MAX_VISIBLE_SELECT_TAGS = 2;

export type UserFiltersValue = typeof DEFAULT_FILTERS;
