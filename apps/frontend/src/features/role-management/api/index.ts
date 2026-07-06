import type {
  Role,
  RoleUser,
  RoleInput,
  RoleDeptBinding,
  RoleMenuBinding,
  RoleUserBinding,
  RoleDataScopeInput,
  RoleMenuTreeSelect,
  RoleDeptTreeSelect,
} from 'src/entities/role/model/types';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { downloadBlobResponse } from 'src/shared/api/download';
import { requestData, isEndpointKey, compactParams } from 'src/shared/api/pagination';

import { roleEndpoints } from 'src/entities/role/api/endpoints';

const NAVBAR_ENDPOINT = '/api/navbar';

export async function createRole(payload: RoleInput) {
  const role = await requestData<Role>(axios.post(roleEndpoints.roles, payload));
  await refreshRoles();
  return role;
}

export async function updateRole(id: string, payload: RoleInput) {
  const role = await requestData<Role>(axios.put(roleEndpoints.role(id), payload));
  await refreshRoles();
  return role;
}

export async function deleteRole(id: string) {
  await axios.delete(roleEndpoints.role(id));
  await refreshRoles();
}

export async function deleteRoles(ids: string[]) {
  await axios.delete(roleEndpoints.rolesBatch, { data: { ids } });
  await refreshRoles();
}

export async function exportRoles(filters: Record<string, string>) {
  const response = await axios.post<Blob>(roleEndpoints.exportRoles, null, {
    params: compactParams(filters),
    responseType: 'blob',
  });
  downloadBlobResponse(response, 'roles.xlsx');
}

export async function updateRoleStatus(id: string, status: string) {
  const role = await requestData<Role>(axios.put(roleEndpoints.status(id), { status }));
  await refreshRoles();
  return role;
}

export async function updateRoleDataScope(id: string, payload: RoleDataScopeInput) {
  const role = await requestData<Role>(axios.put(roleEndpoints.dataScope(id), payload));
  await refreshRoles();
  return role;
}

export function getRoleMenus(id: string) {
  return requestData<RoleMenuBinding>(axios.get(roleEndpoints.roleMenus(id)));
}

export function getRoleMenuTree(id: string) {
  return requestData<RoleMenuTreeSelect>(axios.get(roleEndpoints.roleMenuTreeSelect(id)));
}

export async function updateRoleMenus(id: string, menuIds: string[]) {
  await axios.put(roleEndpoints.roleMenus(id), { menu_ids: menuIds });
  await mutate(NAVBAR_ENDPOINT);
}

export function getRoleDepts(id: string) {
  return requestData<RoleDeptBinding>(axios.get(roleEndpoints.roleDepts(id)));
}

export function getRoleDeptTree(id: string) {
  return requestData<RoleDeptTreeSelect>(axios.get(roleEndpoints.roleDeptTreeSelect(id)));
}

export async function updateRoleDepts(id: string, deptIds: string[]) {
  await axios.put(roleEndpoints.roleDepts(id), { dept_ids: deptIds });
  await mutate(NAVBAR_ENDPOINT);
}

export async function updateRoleUsers(id: string, userIds: string[]) {
  await axios.put<RoleUserBinding>(roleEndpoints.users(id), { user_ids: userIds });
  await refreshRoleUsers(id);
}

export async function deleteRoleUser(id: string, userId: string) {
  await axios.delete(roleEndpoints.roleUser(id, userId));
  await refreshRoleUsers(id);
}

export async function deleteRoleUsers(id: string, userIds: string[]) {
  await axios.delete(roleEndpoints.usersBatch(id), { data: { ids: userIds } });
  await refreshRoleUsers(id);
}

export async function assignRoleUsers(id: string, userIds: string[]) {
  const existing = await requestData<{ items: RoleUser[] }>(
    axios.get(roleEndpoints.users(id), { params: { page: 1, page_size: 1000, allocated: true } })
  );
  const merged = Array.from(new Set([...existing.items.map((user) => user.user_id), ...userIds]));
  await updateRoleUsers(id, merged);
}

async function refreshRoles() {
  await mutate((key) => isEndpointKey(key, roleEndpoints.roles));
  await mutate(NAVBAR_ENDPOINT);
}

async function refreshRoleUsers(roleId: string) {
  await mutate((key) => isEndpointKey(key, roleEndpoints.users(roleId)));
}
