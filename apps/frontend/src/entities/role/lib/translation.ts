import type { Role } from '../model/types';
import type { AdminT } from 'src/shared/ui/admin/common';

export function translatedRoleName(role: Role, t: AdminT) {
  const keyByCode: Record<string, string> = {
    admin: 'roles.admin.name',
    user: 'roles.user.name',
  };

  const key = keyByCode[role.code];

  return key ? t(key) : role.name;
}

export function translatedRoleDescription(role: Role, t: AdminT) {
  const keyByCode: Record<string, string> = {
    admin: 'roles.admin.description',
    user: 'roles.user.description',
  };

  const key = keyByCode[role.code];

  return key ? t(key) : role.description;
}
