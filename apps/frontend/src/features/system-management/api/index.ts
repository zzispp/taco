import type {
  Dept,
  Post,
  DictData,
  DictType,
  DeptInput,
  PostInput,
  ConfigItem,
  ConfigInput,
  DictDataInput,
  DictTypeInput,
  TreeSelectNode,
} from 'src/entities/system';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { requestData, isEndpointKey } from 'src/shared/api/pagination';

import { systemEndpoints } from 'src/entities/system';

export const systemMutations = {
  createDept: crudCreate<Dept, DeptInput>(systemEndpoints.depts),
  updateDept: crudUpdate<Dept, DeptInput>(systemEndpoints.depts, systemEndpoints.dept),
  deleteDept: crudDelete(systemEndpoints.depts, systemEndpoints.dept),
  updateDeptSort,
  updateDeptSorts,
  createPost: crudCreate<Post, PostInput>(systemEndpoints.posts),
  updatePost: crudUpdate<Post, PostInput>(systemEndpoints.posts, systemEndpoints.post),
  deletePost: crudDelete(systemEndpoints.posts, systemEndpoints.post),
  deletePosts: crudDeleteBatch(systemEndpoints.posts, systemEndpoints.postsBatch),
  createDictType: crudCreate<DictType, DictTypeInput>(systemEndpoints.dictTypes),
  updateDictType: crudUpdate<DictType, DictTypeInput>(systemEndpoints.dictTypes, systemEndpoints.dictType),
  deleteDictType: crudDelete(systemEndpoints.dictTypes, systemEndpoints.dictType),
  deleteDictTypes: crudDeleteBatch(systemEndpoints.dictTypes, systemEndpoints.dictTypesBatch),
  refreshDictCache,
  createDictData: crudCreate<DictData, DictDataInput>(systemEndpoints.dictData),
  updateDictData: crudUpdate<DictData, DictDataInput>(systemEndpoints.dictData, systemEndpoints.dictDatum),
  deleteDictData: crudDelete(systemEndpoints.dictData, systemEndpoints.dictDatum),
  deleteDictDataBatch: crudDeleteBatch(systemEndpoints.dictData, systemEndpoints.dictDataBatch),
  createConfig: crudCreate<ConfigItem, ConfigInput>(systemEndpoints.configs),
  updateConfig: crudUpdate<ConfigItem, ConfigInput>(systemEndpoints.configs, systemEndpoints.config),
  deleteConfig: crudDelete(systemEndpoints.configs, systemEndpoints.config),
  deleteConfigs: crudDeleteBatch(systemEndpoints.configs, systemEndpoints.configsBatch),
  refreshConfigCache,
};

function crudCreate<T, I>(collection: string) {
  return async (payload: I) => {
    const item = await requestData<T>(axios.post(collection, payload));
    await mutate((key) => isEndpointKey(key, collection));
    return item;
  };
}

function crudUpdate<T, I>(collection: string, itemEndpoint: (id: string) => string) {
  return async (id: string, payload: I) => {
    const item = await requestData<T>(axios.put(itemEndpoint(id), payload));
    await mutate((key) => isEndpointKey(key, collection));
    return item;
  };
}

function crudDelete(collection: string, itemEndpoint: (id: string) => string) {
  return async (id: string) => {
    await axios.delete(itemEndpoint(id));
    await mutate((key) => isEndpointKey(key, collection));
  };
}

function crudDeleteBatch(collection: string, endpoint: string) {
  return async (ids: string[]) => {
    await axios.delete(endpoint, { data: { ids } });
    await mutate((key) => isEndpointKey(key, collection));
  };
}

async function updateDeptSort(id: string, orderNum: number) {
  const item = await requestData<Dept>(axios.put(systemEndpoints.deptSort(id), { order_num: orderNum }));
  await mutate((key) => isEndpointKey(key, systemEndpoints.depts));
  return item;
}

async function updateDeptSorts(items: { id: string; order_num: number }[]) {
  const depts = await requestData<Dept[]>(axios.put(systemEndpoints.deptSortBatch, { items }));
  await mutate((key) => isEndpointKey(key, systemEndpoints.depts));
  return depts;
}

async function refreshDictCache() {
  await axios.delete(systemEndpoints.dictTypeCache);
}

async function refreshConfigCache() {
  await axios.delete(systemEndpoints.configCache);
}

export async function getDeptTree() {
  return requestData<TreeSelectNode[]>(axios.get(systemEndpoints.deptTreeSelect));
}

export async function getDeptExclude(id: string) {
  return requestData<TreeSelectNode[]>(axios.get(systemEndpoints.deptExclude(id)));
}
