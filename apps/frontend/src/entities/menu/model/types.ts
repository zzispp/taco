export type MenuSection = {
  id: string;
  code: string;
  subheader: string;
  sort_order: number;
  enabled: boolean;
};

export type MenuSectionInput = {
  code: string;
  subheader: string;
  sort_order: number;
  enabled: boolean;
};

export type MenuItem = {
  id: string;
  section_id: string;
  parent_id: string | null;
  code: string;
  title: string;
  path: string;
  icon: string | null;
  caption: string | null;
  deep_match: boolean;
  sort_order: number;
  enabled: boolean;
};

export type MenuItemInput = {
  section_id: string;
  parent_id: string | null;
  code: string;
  title: string;
  path: string;
  icon: string | null;
  caption: string | null;
  deep_match: boolean;
  sort_order: number;
  enabled: boolean;
};
