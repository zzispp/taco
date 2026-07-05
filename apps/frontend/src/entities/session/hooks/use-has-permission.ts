'use client';

import { useAuthContext } from './use-auth-context';

const ALL_PERMISSION = '*:*:*';
const SUPER_ADMIN_ROLE_KEY = 'admin';

export function useHasPermission(permission: string) {
  const { user } = useAuthContext();
  if (!user) {
    return false;
  }

  if (user.system || user.permissions.includes(ALL_PERMISSION)) {
    return true;
  }

  if (user.roles.some((role) => role.role_key === SUPER_ADMIN_ROLE_KEY)) {
    return true;
  }

  return user.permissions.includes(permission);
}
