'use client';

import type { Menu } from '../model/types';
import type { TranslateFn } from 'src/shared/i18n';
import type { NavSectionProps } from 'src/shared/ui/nav-section';

import { CONFIG } from 'src/shared/config';
import { SvgColor } from 'src/shared/ui/svg-color';

// ----------------------------------------------------------------------

export const NAV_ICON_OPTIONS = [
  'icon.dashboard',
  'icon.user',
  'icon.lock',
  'icon.menu',
  'icon.analytics',
  'icon.file',
  'icon.folder',
  'icon.calendar',
  'icon.kanban',
  'icon.mail',
  'icon.chat',
  'icon.blank',
];

export const NAV_ICONS: NonNullable<NavSectionProps['render']>['navIcon'] = {
  'icon.analytics': icon('ic-analytics'),
  'icon.blank': icon('ic-blank'),
  'icon.calendar': icon('ic-calendar'),
  'icon.chat': icon('ic-chat'),
  'icon.dashboard': icon('ic-dashboard'),
  'icon.file': icon('ic-file'),
  'icon.folder': icon('ic-folder'),
  'icon.kanban': icon('ic-kanban'),
  'icon.lock': icon('ic-lock'),
  'icon.mail': icon('ic-mail'),
  'icon.menu': icon('ic-menu-item'),
  'icon.user': icon('ic-user'),
};

export function translatedMenuItem(item: Menu, t: TranslateFn) {
  const keyByPerms: Record<string, string> = {
    'system:dashboard:view': 'nav.dashboard',
    'system:user:list': 'nav.users',
    'system:role:list': 'nav.roles',
    'system:menu:list': 'nav.menus',
    'system:dept:list': 'nav.depts',
    'system:post:list': 'nav.posts',
    'system:dict:list': 'nav.dicts',
    'system:config:list': 'nav.configs',
  };
  const key = item.perms ? keyByPerms[item.perms] : undefined;
  return key ? t(key) : item.menu_name;
}

function icon(name: string) {
  return <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/${name}.svg`} />;
}
