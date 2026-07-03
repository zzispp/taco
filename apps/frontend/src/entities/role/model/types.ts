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

export type RoleApiBinding = {
  api_permission_ids: string[];
};

export type RoleMenuBinding = {
  menu_item_ids: string[];
};
