import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { isEndpointKey } from 'src/shared/api/pagination';

import { noticeEndpoints } from 'src/entities/notice';

export async function markNoticeRead(id: string) {
  await axios.put(noticeEndpoints.read(id));
  await refreshTopNotices();
}

export async function markAllNoticesRead() {
  await axios.put(noticeEndpoints.readAll);
  await refreshTopNotices();
}

async function refreshTopNotices() {
  await mutate((key) => isEndpointKey(key, noticeEndpoints.top));
}
