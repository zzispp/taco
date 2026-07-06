import type { DictDataInput, DictTypeInput } from 'src/entities/system';

export const DEFAULT_TYPE_INPUT: DictTypeInput = {
  dict_name: '',
  dict_type: '',
  status: '0',
  remark: '',
};

export const DEFAULT_DATA_INPUT: DictDataInput = {
  dict_sort: 0,
  dict_label: '',
  dict_value: '',
  dict_type: '',
  css_class: '',
  list_class: 'default',
  is_default: 'N',
  status: '0',
  remark: '',
};

export const DEFAULT_TYPE_FILTERS = {
  dict_name: '',
  dict_type: '',
  status: '',
  begin_time: '',
  end_time: '',
};

export const DEFAULT_DATA_FILTERS = { dict_label: '', status: '' };

export type DictTypeFiltersValue = typeof DEFAULT_TYPE_FILTERS;
export type DictDataFiltersValue = typeof DEFAULT_DATA_FILTERS;
