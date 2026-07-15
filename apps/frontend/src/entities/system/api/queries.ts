import type { ServerDashboard } from '../model/dashboard';
import type { PublicConfigMap } from '../model/public-config';
import type { QueryParams, CursorPageRequest } from 'src/shared/api/pagination';
import type { Dept, Post, DictData, DictType, ConfigItem } from '../model/types';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';
import { useCursorResource } from 'src/shared/api/use-cursor-resource';
import { useCursorCollection } from 'src/shared/api/use-cursor-collection';

import { systemEndpoints } from './endpoints';
import { publicConfigKeys } from '../model/public-config';

export function useServerDashboard() {
  return useSWR<ServerDashboard>(systemEndpoints.dashboard, fetcher, {
    refreshInterval: 5000,
    revalidateOnFocus: false,
  });
}

export function useDepts(params: QueryParams = {}) {
  return useCursorCollection<Dept>({ endpoint: systemEndpoints.depts, params });
}

export function usePosts(request: CursorPageRequest, params: QueryParams = {}) {
  return useCursorResource<Post>({ endpoint: systemEndpoints.posts, request, params });
}

export function useDictTypes(request: CursorPageRequest, params: QueryParams = {}) {
  return useCursorResource<DictType>({
    endpoint: systemEndpoints.dictTypes,
    request,
    params,
  });
}

export function useDictData(request: CursorPageRequest, params: QueryParams = {}) {
  return useCursorResource<DictData>({ endpoint: systemEndpoints.dictData, request, params });
}

export function useConfigs(request: CursorPageRequest, params: QueryParams = {}) {
  return useCursorResource<ConfigItem>({
    endpoint: systemEndpoints.configs,
    request,
    params,
  });
}

export function usePublicConfigs(keys: string[] = publicConfigKeys()) {
  const query = keys.map(encodeURIComponent).join(',');
  return useSWR<PublicConfigMap>(`${systemEndpoints.appConfigs}?keys=${query}`, fetcher, {
    revalidateOnFocus: false,
  });
}
