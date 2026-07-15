import type { SessionUser } from '../model/types';

export type PermissionChecker = (permission: string) => boolean;

const ALL_PERMISSION = '*:*:*';
const SUPER_ADMIN_ROLE_KEY = 'admin';

export function hasSessionPermission(user: SessionUser | null, permission: string): boolean {
  if (!user) return false;
  if (user.permissions.includes(ALL_PERMISSION)) return true;
  if (user.roles.some((role) => role.role_key === SUPER_ADMIN_ROLE_KEY)) return true;
  return user.permissions.includes(permission);
}
