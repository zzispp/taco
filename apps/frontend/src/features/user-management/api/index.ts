import type { UserInput, SystemUser, UserRolesPayload } from 'src/entities/user/model/types';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { requestData, isEndpointKey } from 'src/shared/api/pagination';

import { userEndpoints } from 'src/entities/user/api/endpoints';

export async function createUser(payload: UserInput) {
  const user = await requestData<SystemUser>(axios.post(userEndpoints.users, payload));
  await refreshUsers();
  return user;
}

export async function updateUser(id: string, payload: UserInput) {
  const user = await requestData<SystemUser>(axios.put(userEndpoints.user(id), payload));
  await refreshUsers();
  return user;
}

export async function deleteUser(id: string) {
  await axios.delete(userEndpoints.user(id));
  await refreshUsers();
}

export async function deleteUsers(ids: string[]) {
  await axios.delete(userEndpoints.usersBatch, { data: { ids } });
  await refreshUsers();
}

export async function updateUserStatus(id: string, status: string) {
  const user = await requestData<SystemUser>(axios.put(userEndpoints.status(id), { status }));
  await refreshUsers();
  return user;
}

export async function resetUserPassword(id: string, password: string) {
  await axios.put(userEndpoints.password(id), { password });
}

export function getUserRoles(id: string) {
  return requestData<UserRolesPayload>(axios.get(userEndpoints.roles(id)));
}

export async function updateUserRoles(id: string, roleIds: string[]) {
  const user = await requestData<SystemUser>(axios.put(userEndpoints.roles(id), { role_ids: roleIds }));
  await refreshUsers();
  return user;
}

async function refreshUsers() {
  await mutate((key) => isEndpointKey(key, userEndpoints.users));
}
