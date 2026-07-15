'use client';

import type { Menu } from '../model/types';
import type { TranslateFn } from 'src/shared/i18n';
import type { IconifyName } from 'src/shared/ui/iconify';
import type { NavSectionProps } from 'src/shared/ui/nav-section';

import { CONFIG } from 'src/shared/config';
import { Iconify } from 'src/shared/ui/iconify';
import { SvgColor } from 'src/shared/ui/svg-color';

import { systemMenuItemTranslationKey } from './system-menu-translation';

// ----------------------------------------------------------------------

type NavIconMap = NonNullable<NonNullable<NavSectionProps['render']>['navIcon']>;

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
  'icon.logs',
  'icon.operation-log',
  'icon.login-log',
];

export const NAV_ICONS: NavIconMap = {
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
  'icon.login-log': iconify('solar:user-id-bold'),
  'icon.logs': iconify('solar:bill-list-bold-duotone'),
  'icon.notice': iconify('solar:bell-bing-bold-duotone'),
  'icon.operation-log': iconify('solar:file-text-bold'),
  'icon.post': icon('ic-job'),
  'icon.user': icon('ic-user'),
};

export function translatedMenuItem(item: Menu, t: TranslateFn) {
  const key = systemMenuItemTranslationKey(item.menu_id, item.path);
  return key ? t(key) : item.menu_name;
}

function icon(name: string) {
  return <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/${name}.svg`} />;
}

function iconify(name: IconifyName) {
  return <Iconify icon={name} width={24} />;
}
