import type { Role, RoleUser } from '../model/types';
import type { QueryParams, CursorPageRequest } from 'src/shared/api/pagination';

import { useCursorResource } from 'src/shared/api/use-cursor-resource';

import { roleEndpoints } from './endpoints';

export function useRoles(request: CursorPageRequest, params: QueryParams = {}) {
  return useCursorResource<Role>({ endpoint: roleEndpoints.roles, request, params });
}

export function useRoleUsers(
  roleId: string | null,
  request: CursorPageRequest,
  params: QueryParams = {}
) {
  return useCursorResource<RoleUser>({
    endpoint: roleId ? roleEndpoints.users(roleId) : '',
    request,
    params,
  });
}
