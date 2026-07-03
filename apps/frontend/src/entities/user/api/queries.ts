import type { SystemUser } from '../model/types';

import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { userEndpoints } from './endpoints';

export function useUsers(page: number, pageSize: number) {
  return usePagedResource<SystemUser>(userEndpoints.users, page, pageSize);
}
