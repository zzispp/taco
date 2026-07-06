import type {
  UserInput,
  SystemUser,
  UserImportResult,
  UserRolesPayload,
} from 'src/entities/user/model/types';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { downloadBlobResponse } from 'src/shared/api/download';
import { requestData, isEndpointKey, compactParams } from 'src/shared/api/pagination';

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

export async function exportUsers(filters: Record<string, string>) {
  const response = await axios.post<Blob>(userEndpoints.exportUsers, null, {
    params: compactParams(filters),
    responseType: 'blob',
  });
  downloadBlobResponse(response, 'users.xlsx');
}

export async function importUsers(file: File, updateSupport: boolean) {
  const form = new FormData();
  form.append('file', file);
  form.append('update_support', String(updateSupport));
  const result = await requestData<UserImportResult>(
    axios.post(userEndpoints.importUsers, form, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  );
  await refreshUsers();
  return result;
}

export async function downloadUserImportTemplate() {
  const response = await axios.post<Blob>(userEndpoints.importTemplate, null, {
    responseType: 'blob',
  });
  downloadBlobResponse(response, 'user_template.xlsx');
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
  const user = await requestData<SystemUser>(
    axios.put(userEndpoints.roles(id), { role_ids: roleIds })
  );
  await refreshUsers();
  return user;
}

async function refreshUsers() {
  await mutate((key) => isEndpointKey(key, userEndpoints.users));
}
