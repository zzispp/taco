import type { SystemUser, ProfileInput } from 'src/entities/user';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { requestData } from 'src/shared/api/pagination';

import { userEndpoints } from 'src/entities/user';

export async function updateAccountProfile(payload: ProfileInput) {
  const user = await requestData<SystemUser>(axios.put(userEndpoints.accountProfile, payload));
  await refreshAccount();
  return user;
}

export async function changeAccountPassword(oldPassword: string, newPassword: string) {
  await axios.put(userEndpoints.accountPassword, {
    old_password: oldPassword,
    new_password: newPassword,
  });
}

export async function uploadAccountAvatar(file: Blob, filename: string) {
  const form = new FormData();
  form.append('avatarfile', file, filename);
  const result = await requestData<{ img_url: string; user: SystemUser }>(
    axios.post(userEndpoints.accountAvatar, form, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  );
  await refreshAccount();
  return result;
}

async function refreshAccount() {
  await mutate(userEndpoints.accountProfile);
  await mutate('/api/auth/me');
}
