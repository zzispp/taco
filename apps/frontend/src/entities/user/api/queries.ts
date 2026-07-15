import type { QueryParams, CursorPageRequest } from 'src/shared/api/pagination';
import type { SystemUser, AccountProfile, UserFormOptions } from '../model/types';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';
import { useCursorResource } from 'src/shared/api/use-cursor-resource';

import { userEndpoints } from './endpoints';

export function useUsers(request: CursorPageRequest, params: QueryParams = {}) {
  return useCursorResource<SystemUser>({ endpoint: userEndpoints.users, request, params });
}

export function useUserFormOptions() {
  return useSWR<UserFormOptions>(userEndpoints.formOptions, fetcher, { revalidateOnFocus: false });
}

export function useAccountProfile() {
  return useSWR<AccountProfile>(userEndpoints.accountProfile, fetcher, {
    revalidateOnFocus: false,
  });
}
