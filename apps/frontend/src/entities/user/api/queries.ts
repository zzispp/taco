import type { QueryParams } from 'src/shared/api/pagination';
import type { SystemUser, AccountProfile, UserFormOptions } from '../model/types';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';
import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { userEndpoints } from './endpoints';

export function useUsers(page: number, pageSize: number, params: QueryParams = {}) {
  return usePagedResource<SystemUser>({ endpoint: userEndpoints.users, page, pageSize, params });
}

export function useUserFormOptions() {
  return useSWR<UserFormOptions>(userEndpoints.formOptions, fetcher, { revalidateOnFocus: false });
}

export function useAccountProfile() {
  return useSWR<AccountProfile>(userEndpoints.accountProfile, fetcher, {
    revalidateOnFocus: false,
  });
}
