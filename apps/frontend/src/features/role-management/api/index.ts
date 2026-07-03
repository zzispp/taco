import type { Role, RoleInput, RoleApiBinding, RoleMenuBinding } from 'src/entities/role/model/types';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { requestData, isEndpointKey } from 'src/shared/api/pagination';

import { roleEndpoints } from 'src/entities/role/api/endpoints';

const NAVBAR_ENDPOINT = '/api/navbar';

export async function createRole(payload: RoleInput) {
  const role = await requestData<Role>(axios.post(roleEndpoints.roles, payload));
  await mutate((key) => isEndpointKey(key, roleEndpoints.roles));
  await mutate(NAVBAR_ENDPOINT);
  return role;
}

export async function updateRole(code: string, payload: RoleInput) {
  const role = await requestData<Role>(axios.put(roleEndpoints.role(code), payload));
  await mutate((key) => isEndpointKey(key, roleEndpoints.roles));
  await mutate(NAVBAR_ENDPOINT);
  return role;
}

export async function deleteRole(code: string) {
  await axios.delete(roleEndpoints.role(code));
  await mutate((key) => isEndpointKey(key, roleEndpoints.roles));
  await mutate(NAVBAR_ENDPOINT);
}

export function getRoleApis(code: string) {
  return requestData<RoleApiBinding>(axios.get(roleEndpoints.roleApis(code)));
}

export async function updateRoleApis(code: string, apiPermissionIds: string[]) {
  await axios.put(roleEndpoints.roleApis(code), { api_permission_ids: apiPermissionIds });
  await mutate(NAVBAR_ENDPOINT);
}

export function getRoleMenus(code: string) {
  return requestData<RoleMenuBinding>(axios.get(roleEndpoints.roleMenus(code)));
}

export async function updateRoleMenus(code: string, menuItemIds: string[]) {
  await axios.put(roleEndpoints.roleMenus(code), { menu_item_ids: menuItemIds });
  await mutate(NAVBAR_ENDPOINT);
}
