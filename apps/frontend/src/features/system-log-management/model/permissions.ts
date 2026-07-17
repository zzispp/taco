export const SYSTEM_LOG_PERMISSION = {
  LIST: 'system:systemlog:list',
  QUERY: 'system:systemlog:query',
  REMOVE: 'system:systemlog:remove',
  EXPORT: 'system:systemlog:export',
} as const;

export function systemLogCapabilities(hasPermission: (permission: string) => boolean) {
  return {
    list: hasPermission(SYSTEM_LOG_PERMISSION.LIST),
    query: hasPermission(SYSTEM_LOG_PERMISSION.QUERY),
    remove: hasPermission(SYSTEM_LOG_PERMISSION.REMOVE),
    export: hasPermission(SYSTEM_LOG_PERMISSION.EXPORT),
  } as const;
}
