import type { MenuInput } from 'src/entities/menu';

export const DEFAULT_FORM: MenuInput = {
  menu_name: '',
  parent_id: '0',
  order_num: 0,
  path: '',
  component: '',
  query: '',
  route_name: '',
  is_frame: false,
  is_cache: false,
  menu_type: 'C',
  visible: '0',
  status: '0',
  perms: '',
  icon: 'icon.menu',
  remark: '',
};

export const DEFAULT_FILTERS = { menu_name: '', status: '' };
