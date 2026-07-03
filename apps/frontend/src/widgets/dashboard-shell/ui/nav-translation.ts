'use client';

import type { useTranslate } from 'src/shared/i18n/use-locales';
import type { NavSectionProps } from 'src/shared/ui/nav-section';

// ----------------------------------------------------------------------

type NavData = NavSectionProps['data'];
type NavItem = NavData[number]['items'][number];
type TranslateFn = ReturnType<typeof useTranslate>['t'];

const SECTION_KEY_BY_CODE: Record<string, string> = {
  overview: 'nav.overview',
  account: 'nav.account',
  resources: 'nav.resources',
  system_management: 'nav.systemManagement',
};

const SECTION_KEY_BY_TITLE: Record<string, string> = {
  Management: 'nav.systemManagement',
  Overview: 'nav.overview',
  Resources: 'nav.resources',
  'System Management': 'nav.systemManagement',
  '概览': 'nav.overview',
  '系统管理': 'nav.systemManagement',
  '资源': 'nav.resources',
};

const ITEM_KEY_BY_CODE: Record<string, string> = {
  dashboard_home: 'nav.dashboard',
  system_management: 'nav.systemManagement',
  admin_users: 'nav.users',
  admin_roles: 'nav.roles',
  admin_apis: 'nav.apis',
  admin_menus: 'nav.menus',
};

const ITEM_KEY_BY_PATH: Record<string, string> = {
  '/dashboard': 'nav.dashboard',
  '/dashboard/admin': 'nav.systemManagement',
  '/dashboard/admin/users': 'nav.users',
  '/dashboard/admin/roles': 'nav.roles',
  '/dashboard/admin/apis': 'nav.apis',
  '/dashboard/admin/menus': 'nav.menus',
};

export function translateNavData(data: NavData, t: TranslateFn): NavData {
  return data.map((section) => ({
    ...section,
    subheader: translateNavSection(section, t),
    items: section.items.map((item) => translateNavItem(item, t)),
  }));
}

function translateNavSection(section: NavData[number], t: TranslateFn) {
  const key =
    codeKey(section.code, SECTION_KEY_BY_CODE) ??
    (section.subheader ? SECTION_KEY_BY_TITLE[section.subheader] : undefined);

  return key ? t(key) : section.subheader;
}

function translateNavItem(item: NavItem, t: TranslateFn): NavItem {
  return {
    ...item,
    title: translateNavItemTitle(item, t),
    caption: item.caption ? translateNavCaption(item.caption, t) : item.caption,
    children: item.children?.map((child) => translateNavItem(child, t)),
  };
}

function translateNavItemTitle(item: NavItem, t: TranslateFn) {
  const key = codeKey(item.code, ITEM_KEY_BY_CODE) ?? ITEM_KEY_BY_PATH[item.path];
  return key ? t(key) : item.title;
}

function translateNavCaption(caption: string, t: TranslateFn) {
  if (caption === 'Custom keyboard shortcuts.') {
    return t('nav.customKeyboardShortcuts');
  }

  return caption;
}

function codeKey(code: string | undefined, keys: Record<string, string>) {
  return code ? keys[code] : undefined;
}
