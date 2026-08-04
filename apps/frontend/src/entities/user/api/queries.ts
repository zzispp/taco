import type { SystemUser, AccountProfile } from '../model/types';
import type { QueryParams, CursorPageRequest } from 'src/shared/api/pagination';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';
import { useCursorResource } from 'src/shared/api/use-cursor-resource';

import { userEndpoints } from './endpoints';

export function useUsers(request: CursorPageRequest, params: QueryParams = {}) {
  return useCursorResource<SystemUser>({ endpoint: userEndpoints.users, request, params });
}

export function useAccountProfile() {
  return useSWR<AccountProfile>(userEndpoints.accountProfile, fetcher, {
    revalidateOnFocus: false,
  });
}
