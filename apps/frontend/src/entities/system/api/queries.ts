import type { QueryParams } from 'src/shared/api/pagination';
import type { Dept, Post, DictData, DictType, ConfigItem } from '../model/types';

import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { systemEndpoints } from './endpoints';

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
