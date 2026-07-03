import type { Role } from '../model/types';

import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { roleEndpoints } from './endpoints';

export function useRoles(page: number, pageSize: number) {
  return usePagedResource<Role>(roleEndpoints.roles, page, pageSize);
}
