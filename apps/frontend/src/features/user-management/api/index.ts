import type { UserInput, SystemUser } from 'src/entities/user/model/types';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { requestData, isEndpointKey } from 'src/shared/api/pagination';

import { userEndpoints } from 'src/entities/user/api/endpoints';

export async function createUser(payload: UserInput) {
  const user = await requestData<SystemUser>(axios.post(userEndpoints.users, payload));
  await mutate((key) => isEndpointKey(key, userEndpoints.users));
  return user;
}

export async function updateUser(id: string, payload: UserInput) {
  const user = await requestData<SystemUser>(axios.put(userEndpoints.user(id), payload));
  await mutate((key) => isEndpointKey(key, userEndpoints.users));
  return user;
}

export async function deleteUser(id: string) {
  await axios.delete(userEndpoints.user(id));
  await mutate((key) => isEndpointKey(key, userEndpoints.users));
}
