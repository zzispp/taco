'use client';

import type { useTranslate } from 'src/shared/i18n/use-locales';
import type { NavSectionProps } from 'src/shared/ui/nav-section';

import { systemMenuItemTranslationKey, systemMenuSectionTranslationKey } from 'src/entities/menu';

// ----------------------------------------------------------------------

type NavData = NavSectionProps['data'];
type NavItem = NavData[number]['items'][number];
type TranslateFn = ReturnType<typeof useTranslate>['t'];

export function translateNavData(data: NavData, t: TranslateFn): NavData {
  return data.map((section) => ({
    ...section,
    subheader: translateNavSection(section, t),
    items: section.items.map((item) => translateNavItem(item, t)),
  }));
}

function translateNavSection(section: NavData[number], t: TranslateFn) {
  const key = systemMenuSectionTranslationKey(section.code);
  return key ? t(key) : section.subheader;
}

function translateNavItem(item: NavItem, t: TranslateFn): NavItem {
  return {
    ...item,
    title: translateNavItemTitle(item, t),
    children: item.children?.map((child) => translateNavItem(child, t)),
  };
}

function translateNavItemTitle(item: NavItem, t: TranslateFn) {
  const key = systemMenuItemTranslationKey(item.code, item.path);
  return key ? t(key) : item.title;
}
