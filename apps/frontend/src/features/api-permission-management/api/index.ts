import type { ApiPermission, ApiPermissionInput } from 'src/entities/api-permission/model/types';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { requestData, isEndpointKey } from 'src/shared/api/pagination';

import { apiPermissionEndpoints } from 'src/entities/api-permission/api/endpoints';

export async function createApi(payload: ApiPermissionInput) {
  const api = await requestData<ApiPermission>(axios.post(apiPermissionEndpoints.apis, payload));
  await mutate((key) => isEndpointKey(key, apiPermissionEndpoints.apis));
  return api;
}

export async function updateApi(id: string, payload: ApiPermissionInput) {
  const api = await requestData<ApiPermission>(axios.put(apiPermissionEndpoints.api(id), payload));
  await mutate((key) => isEndpointKey(key, apiPermissionEndpoints.apis));
  return api;
}

export async function deleteApi(id: string) {
  await axios.delete(apiPermissionEndpoints.api(id));
  await mutate((key) => isEndpointKey(key, apiPermissionEndpoints.apis));
}
