export type TreeSelectNode = {
  id: string;
  label: string;
  parent_id: string;
  disabled: boolean;
  children: TreeSelectNode[];
};

export type Dept = {
  dept_id: string;
  parent_id: string;
  ancestors: string;
  dept_name: string;
  order_num: number;
  leader: string | null;
  phone: string | null;
  email: string | null;
  status: string;
  create_time: string;
};

export type DeptInput = Omit<Dept, 'dept_id' | 'ancestors' | 'create_time'>;

export type Post = {
  post_id: string;
  post_code: string;
  post_name: string;
  post_sort: number;
  status: string;
  remark: string | null;
  create_time: string;
};

export type PostInput = Omit<Post, 'post_id' | 'create_time'>;

export type DictType = {
  dict_id: string;
  dict_name: string;
  dict_type: string;
  status: string;
  remark: string | null;
  create_time: string;
};

export type DictTypeInput = Omit<DictType, 'dict_id' | 'create_time'>;

export type DictData = {
  dict_code: string;
  dict_sort: number;
  dict_label: string;
  dict_value: string;
  dict_type: string;
  css_class: string | null;
  list_class: string | null;
  is_default: string;
  status: string;
  remark: string | null;
  create_time: string;
};

export type DictDataInput = Omit<DictData, 'dict_code' | 'create_time'>;

export type ConfigItem = {
  config_id: string;
  config_name: string;
  config_key: string;
  config_value: string;
  config_type: string;
  remark: string | null;
  create_time: string;
};

export type ConfigInput = Omit<ConfigItem, 'config_id' | 'create_time'>;
