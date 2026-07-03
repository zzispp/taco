import type { AdminT } from 'src/shared/ui/admin/common';
import type { MenuItem, MenuSection } from '../model/types';

export function translatedMenuSection(section: MenuSection, t: AdminT) {
  const keyByCode: Record<string, string> = {
    overview: 'nav.overview',
    account: 'nav.account',
    resources: 'nav.resources',
    system_management: 'nav.systemManagement',
  };

  const key = keyByCode[section.code];

  return key ? t(key) : section.subheader;
}

export function translatedMenuItem(item: MenuItem, t: AdminT) {
  const keyByCode: Record<string, string> = {
    dashboard_home: 'nav.dashboard',
    admin_users: 'nav.users',
    admin_roles: 'nav.roles',
    admin_apis: 'nav.apis',
    admin_menus: 'nav.menus',
  };

  const key = keyByCode[item.code];

  return key ? t(key) : item.title;
}
