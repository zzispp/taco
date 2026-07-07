import type { Menu, MenuInput } from 'src/entities/menu';
import type { IconifyName } from 'src/shared/ui/iconify';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { useTranslate } from 'src/shared/i18n/use-locales';

type Translate = ReturnType<typeof useTranslate>['t'];

export type MenuRowView = { menu: Menu; level: number; childCount: number };
export type ParentMenuNode = {
  id: string;
  label: string;
  disabled: boolean;
  children: ParentMenuNode[];
};

export function toInput(menu: Menu): MenuInput {
  return {
    menu_name: menu.menu_name,
    parent_id: menu.parent_id,
    order_num: menu.order_num,
    path: menu.path,
    component: menu.component,
    query: menu.query,
    route_name: menu.route_name,
    is_frame: menu.is_frame,
    is_cache: menu.is_cache,
    menu_type: menu.menu_type,
    visible: menu.visible,
    status: menu.status,
    perms: menu.perms,
    icon: menu.icon,
    remark: menu.remark,
  };
}

export function menuTypeLabel(value: string, t: Translate) {
  if (value === 'M') return t('menuType.directory');
  if (value === 'F') return t('menuType.button');
  return t('menuType.menu');
}

export function menuHead(t: Translate): TableHeadCellProps[] {
  return [
    { id: 'menu_name', label: t('fields.menuName') },
    { id: 'menu_type', label: t('fields.menuType') },
    { id: 'order_num', label: t('fields.orderNum') },
    { id: 'path', label: t('fields.path') },
    { id: 'component', label: t('fields.component') },
    { id: 'perms', label: t('fields.perms') },
    { id: 'visible', label: t('fields.visible') },
    { id: 'status', label: t('common.status') },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 132 },
  ];
}

export function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}

export function menuIcon(icon: string): IconifyName {
  return (icon.startsWith('solar:') ? icon : 'solar:list-bold') as IconifyName;
}

export function flattenMenuRows(menus: Menu[], expanded: string[]) {
  return walkMenus({ menus, parentId: '0', level: 0, expanded });
}

type WalkMenusOptions = {
  menus: Menu[];
  parentId: string;
  level: number;
  expanded: string[];
};

function walkMenus(options: WalkMenusOptions): MenuRowView[] {
  const { menus, parentId, level, expanded } = options;
  return childMenus(menus, parentId).flatMap((menu) => [
    { menu, level, childCount: childMenus(menus, menu.menu_id).length },
    ...(expanded.includes(menu.menu_id)
      ? walkMenus({ menus, parentId: menu.menu_id, level: level + 1, expanded })
      : []),
  ]);
}

export function parentMenuTree(menus: Menu[], editingId?: string): ParentMenuNode[] {
  return buildParentMenuNodes(
    menus.filter((menu) => menu.menu_type !== 'F' && menu.menu_id !== editingId),
    '0'
  );
}

function buildParentMenuNodes(menus: Menu[], parentId: string): ParentMenuNode[] {
  return childMenus(menus, parentId).map((menu) => ({
    id: menu.menu_id,
    label: menu.menu_name,
    disabled: menu.status !== '0',
    children: buildParentMenuNodes(menus, menu.menu_id),
  }));
}

function childMenus(menus: Menu[], parentId: string) {
  return menus.filter((menu) => menu.parent_id === parentId).sort(byOrderNum);
}

function byOrderNum(a: Menu, b: Menu) {
  return a.order_num - b.order_num;
}
