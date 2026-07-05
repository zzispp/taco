import type { Role, RoleUser } from '../model/types';
import type { QueryParams } from 'src/shared/api/pagination';

import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { roleEndpoints } from './endpoints';

export function useRoles(page: number, pageSize: number, params: QueryParams = {}) {
  return usePagedResource<Role>(roleEndpoints.roles, page, pageSize, params);
}

export function useRoleUsers(
  roleId: string | null,
  page: number,
  pageSize: number,
  params: QueryParams = {}
) {
  return usePagedResource<RoleUser>(roleId ? roleEndpoints.users(roleId) : '', page, pageSize, params);
}
