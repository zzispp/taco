export type Menu = {
  menu_id: string;
  menu_name: string;
  parent_id: string;
  order_num: number;
  path: string;
  component: string | null;
  query: string | null;
  route_name: string;
  is_frame: boolean;
  is_cache: boolean;
  menu_type: string;
  visible: string;
  status: string;
  perms: string | null;
  icon: string;
  remark: string | null;
};

export type MenuInput = {
  menu_name: string;
  parent_id: string;
  order_num: number;
  path: string;
  component: string | null;
  query: string | null;
  route_name: string;
  is_frame: boolean;
  is_cache: boolean;
  menu_type: string;
  visible: string;
  status: string;
  perms: string | null;
  icon: string;
  remark: string | null;
};
