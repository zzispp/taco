import type { Role, RoleSummary } from '../model/types';

export function translatedRoleName(role: RoleSummary) {
  return role.role_name;
}

export function translatedRoleDescription(role: Role) {
  return role.remark ?? '';
}
