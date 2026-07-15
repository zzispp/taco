'use client';

import { useCallback } from 'react';

import { useAuthContext } from './use-auth-context';
import { hasSessionPermission } from '../lib/permissions';

export function useHasPermission(permission: string) {
  return usePermissionChecker()(permission);
}

export function usePermissionChecker() {
  const { user } = useAuthContext();
  return useCallback((permission: string) => hasSessionPermission(user, permission), [user]);
}
