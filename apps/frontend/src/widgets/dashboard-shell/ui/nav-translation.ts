'use client';

import type { useTranslate } from 'src/shared/i18n/use-locales';
import type { NavSectionProps } from 'src/shared/ui/nav-section';

// ----------------------------------------------------------------------

type NavData = NavSectionProps['data'];
type NavItem = NavData[number]['items'][number];
type TranslateFn = ReturnType<typeof useTranslate>['t'];

const SECTION_KEY_BY_CODE: Record<string, string> = {
  account: 'nav.account',
  resources: 'nav.resources',
  system_management: 'nav.systemManagement',
  system_monitor: 'nav.systemMonitor',
  '1': 'nav.systemManagement',
  '3': 'nav.systemMonitor',
  '4': 'nav.overview',
};

const SECTION_KEY_BY_TITLE: Record<string, string> = {
  Management: 'nav.systemManagement',
  Resources: 'nav.resources',
  'System Management': 'nav.systemManagement',
  'System Monitor': 'nav.systemMonitor',
  系统管理: 'nav.systemManagement',
  系统监控: 'nav.systemMonitor',
  概览: 'nav.overview',
  概覽: 'nav.overview',
};

const ITEM_KEY_BY_CODE: Record<string, string> = {
  '2': 'nav.dashboard',
  '100': 'nav.users',
  '101': 'nav.roles',
  '102': 'nav.menus',
  '103': 'nav.depts',
  '104': 'nav.posts',
  '105': 'nav.dicts',
  '106': 'nav.configs',
  '107': 'nav.online',
  '108': 'nav.jobs',
  '109': 'nav.jobLogs',
  '110': 'nav.notices',
};

const ITEM_KEY_BY_PATH: Record<string, string> = {
  '/dashboard': 'nav.dashboard',
  '/dashboard/admin': 'nav.systemManagement',
  '/dashboard/admin/users': 'nav.users',
  '/dashboard/admin/roles': 'nav.roles',
  '/dashboard/admin/menus': 'nav.menus',
  '/dashboard/admin/depts': 'nav.depts',
  '/dashboard/admin/posts': 'nav.posts',
  '/dashboard/admin/dicts': 'nav.dicts',
  '/dashboard/admin/configs': 'nav.configs',
  '/dashboard/admin/online': 'nav.online',
  '/dashboard/admin/notices': 'nav.notices',
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
