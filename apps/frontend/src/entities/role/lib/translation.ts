import type { TranslateFn } from 'src/shared/i18n';
import type { Role, RoleSummary } from '../model/types';

export function translatedRoleName(role: RoleSummary, t: TranslateFn) {
  const keyByCode: Record<string, string> = {
    admin: 'roles.admin.name',
    common: 'roles.common.name',
  };

  const key = keyByCode[role.role_key];

  return key ? t(key) : role.role_name;
}

export function translatedRoleDescription(role: Role, t: TranslateFn) {
  const keyByCode: Record<string, string> = {
    admin: 'roles.admin.description',
    common: 'roles.common.description',
  };

  const key = keyByCode[role.role_key];

  return key ? t(key) : (role.remark ?? '');
}
