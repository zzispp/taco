import type { Menu } from '../model/types';
import type { AdminT } from 'src/shared/ui/admin/common';

export function translatedMenuItem(item: Menu, t: AdminT) {
  const keyByPerms: Record<string, string> = {
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
