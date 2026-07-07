import type { ServerDashboard } from '../model/dashboard';
import type { QueryParams } from 'src/shared/api/pagination';
import type { PublicConfigMap } from '../model/public-config';
import type { Dept, Post, DictData, DictType, ConfigItem } from '../model/types';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';
import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { systemEndpoints } from './endpoints';
import { publicConfigKeys } from '../model/public-config';

export function useServerDashboard() {
  return useSWR<ServerDashboard>(systemEndpoints.dashboard, fetcher, {
    refreshInterval: 5000,
    revalidateOnFocus: false,
  });
}

export function useDepts(page: number, pageSize: number, params: QueryParams = {}) {
  return usePagedResource<Dept>(systemEndpoints.depts, page, pageSize, params);
}

export function usePosts(page: number, pageSize: number, params: QueryParams = {}) {
  return usePagedResource<Post>(systemEndpoints.posts, page, pageSize, params);
}

export function useDictTypes(page: number, pageSize: number, params: QueryParams = {}) {
  return usePagedResource<DictType>(systemEndpoints.dictTypes, page, pageSize, params);
}

export function useDictData(page: number, pageSize: number, params: QueryParams = {}) {
  return usePagedResource<DictData>(systemEndpoints.dictData, page, pageSize, params);
}

export function useConfigs(page: number, pageSize: number, params: QueryParams = {}) {
  return usePagedResource<ConfigItem>(systemEndpoints.configs, page, pageSize, params);
}

export function usePublicConfigs(keys: string[] = publicConfigKeys()) {
  const query = keys.map(encodeURIComponent).join(',');
  return useSWR<PublicConfigMap>(`${systemEndpoints.appConfigs}?keys=${query}`, fetcher, {
    revalidateOnFocus: false,
  });
}
