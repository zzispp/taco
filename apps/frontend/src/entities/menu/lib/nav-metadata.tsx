'use client';

import type { Menu } from '../model/types';
import type { TranslateFn } from 'src/shared/i18n';
import type { IconifyName } from 'src/shared/ui/iconify';
import type { NavSectionProps } from 'src/shared/ui/nav-section';

import { CONFIG } from 'src/shared/config';
import { paths } from 'src/shared/routes/paths';
import { Iconify } from 'src/shared/ui/iconify';
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
  'icon.monitor',
  'icon.calendar',
  'icon.kanban',
  'icon.mail',
  'icon.chat',
  'icon.blank',
  'icon.dept',
  'icon.post',
  'icon.dict',
  'icon.config',
  'icon.online',
  'icon.job',
  'icon.job-log',
  'icon.notice',
];

export const NAV_ICONS: NonNullable<NavSectionProps['render']>['navIcon'] = {
  'icon.analytics': icon('ic-analytics'),
  'icon.blank': icon('ic-blank'),
  'icon.calendar': icon('ic-calendar'),
  'icon.chat': icon('ic-chat'),
  'icon.config': iconify('solar:settings-bold-duotone'),
  'icon.dashboard': icon('ic-dashboard'),
  'icon.dept': iconify('solar:users-group-rounded-bold-duotone'),
  'icon.dict': iconify('solar:notebook-bold-duotone'),
  'icon.file': icon('ic-file'),
  'icon.folder': icon('ic-folder'),
  'icon.kanban': icon('ic-kanban'),
  'icon.lock': icon('ic-lock'),
  'icon.mail': icon('ic-mail'),
  'icon.menu': icon('ic-menu-item'),
  'icon.monitor': iconify('solar:monitor-bold'),
  'icon.online': iconify('solar:monitor-bold'),
  'icon.job': iconify('solar:calendar-date-bold'),
  'icon.job-log': iconify('solar:bill-list-bold-duotone'),
  'icon.notice': iconify('solar:bell-bing-bold-duotone'),
  'icon.post': icon('ic-job'),
  'icon.user': icon('ic-user'),
};

export function translatedMenuItem(item: Menu, t: TranslateFn) {
  const keyByPath: Record<string, string> = {
    [paths.dashboard.overview]: 'nav.overview',
    [paths.dashboard.admin.root]: 'nav.systemManagement',
    [paths.dashboard.monitor]: 'nav.systemMonitor',
    [paths.dashboard.admin.jobs]: 'nav.jobs',
    [paths.dashboard.admin.jobLogs]: 'nav.jobLogs',
    [paths.dashboard.admin.notices]: 'nav.notices',
  };
  const keyByPerms: Record<string, string> = {
    'system:dashboard:view': 'nav.dashboard',
    'system:user:list': 'nav.users',
    'system:role:list': 'nav.roles',
    'system:menu:list': 'nav.menus',
    'system:dept:list': 'nav.depts',
    'system:post:list': 'nav.posts',
    'system:dict:list': 'nav.dicts',
    'system:config:list': 'nav.configs',
    'system:online:list': 'nav.online',
    'system:notice:list': 'nav.notices',
  };
  const key = keyByPath[item.path] ?? (item.perms ? keyByPerms[item.perms] : undefined);
  return key ? t(key) : item.menu_name;
}

function icon(name: string) {
  return <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/${name}.svg`} />;
}

function iconify(name: IconifyName) {
  return <Iconify icon={name} width={24} />;
}
