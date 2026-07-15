export const AUDIT_PERMISSION = {
  OPERATION_LIST: 'system:operlog:list',
  OPERATION_QUERY: 'system:operlog:query',
  OPERATION_REMOVE: 'system:operlog:remove',
  OPERATION_EXPORT: 'system:operlog:export',
  LOGIN_LIST: 'system:logininfor:list',
  LOGIN_REMOVE: 'system:logininfor:remove',
  LOGIN_EXPORT: 'system:logininfor:export',
  LOGIN_UNLOCK: 'system:logininfor:unlock',
} as const;

export function auditLogCapabilities(hasPermission: (permission: string) => boolean) {
  return {
    operation: {
      list: hasPermission(AUDIT_PERMISSION.OPERATION_LIST),
      query: hasPermission(AUDIT_PERMISSION.OPERATION_QUERY),
      remove: hasPermission(AUDIT_PERMISSION.OPERATION_REMOVE),
      export: hasPermission(AUDIT_PERMISSION.OPERATION_EXPORT),
    },
    login: {
      list: hasPermission(AUDIT_PERMISSION.LOGIN_LIST),
      remove: hasPermission(AUDIT_PERMISSION.LOGIN_REMOVE),
      export: hasPermission(AUDIT_PERMISSION.LOGIN_EXPORT),
      unlock: hasPermission(AUDIT_PERMISSION.LOGIN_UNLOCK),
    },
  } as const;
}

export function loginLogSelectionActions(
  selected: readonly string[],
  canRemove: boolean,
  canUnlock: boolean
) {
  return {
    canDelete: canRemove && selected.length > 0,
    canUnlock: canUnlock && selected.length === 1,
  } as const;
}
