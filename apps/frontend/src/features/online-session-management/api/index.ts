import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { isEndpointKey } from 'src/shared/api/pagination';

import { onlineSessionEndpoints } from 'src/entities/online-session';

export async function forceLogoutOnlineSession(tokenId: string) {
  await axios.delete(onlineSessionEndpoints.forceLogout(tokenId));
  await refreshOnlineSessions();
}

async function refreshOnlineSessions() {
  await mutate((key) => isEndpointKey(key, onlineSessionEndpoints.list));
}
