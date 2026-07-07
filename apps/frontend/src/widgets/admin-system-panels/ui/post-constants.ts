import type { PostInput } from 'src/entities/system';

export const DEFAULT_POST_INPUT: PostInput = {
  post_code: '',
  post_name: '',
  post_sort: 0,
  status: '0',
  remark: '',
};

export const DEFAULT_POST_FILTERS = { post_name: '', post_code: '', status: '' };
