import type { ApiPermission } from '../model/types';

import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { apiPermissionEndpoints } from './endpoints';

export function useApis(page: number, pageSize: number) {
  return usePagedResource<ApiPermission>(apiPermissionEndpoints.apis, page, pageSize);
}
