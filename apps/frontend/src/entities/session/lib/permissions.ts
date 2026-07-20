import type { SessionUser } from '../model/types';

export type PermissionChecker = (permission: string) => boolean;

const WILDCARD_PERMISSION = '*:*:*';

export function hasSessionPermission(user: SessionUser | null, permission: string): boolean {
  if (!user) return false;
  if (user.is_installation_owner) return true;
  if (permission === WILDCARD_PERMISSION) return false;
  return user.permissions.includes(permission);
}
