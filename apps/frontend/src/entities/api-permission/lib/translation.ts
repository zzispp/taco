import type { ApiPermission } from '../model/types';
import type { AdminT } from 'src/shared/ui/admin/common';

export function translatedApiName(api: ApiPermission, t: AdminT) {
  const translated = t(`apiPermissionNames.${api.code}`);

  return translated === `apiPermissionNames.${api.code}` ? api.name : translated;
}

export function translatedApiGroup(group: string, t: AdminT) {
  if (!group) {
    return '-';
  }

  const key = `apiGroups.${group.toLowerCase()}`;
  const translated = t(key);

  return translated === key ? group : translated;
}
