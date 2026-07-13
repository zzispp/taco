import type { Notice, NoticeInput } from 'src/entities/notice';

import { mutate } from 'swr';

import axios from 'src/shared/api/http-client';
import { requestData, isEndpointKey } from 'src/shared/api/pagination';

import { noticeEndpoints } from 'src/entities/notice';

export async function createNotice(payload: NoticeInput) {
  const notice = await requestData<Notice>(axios.post(noticeEndpoints.notices, payload));
  await refreshNoticeCollections();
  return notice;
}

export async function updateNotice(id: string, payload: NoticeInput) {
  const notice = await requestData<Notice>(axios.put(noticeEndpoints.notice(id), payload));
  await refreshNoticeCollections();
  await mutate((key) => isEndpointKey(key, noticeEndpoints.notice(id)));
  return notice;
}

export async function deleteNotice(id: string) {
  await axios.delete(noticeEndpoints.notice(id));
  await refreshNoticeCollections();
  await clearNoticeDetails([id]);
}

export async function deleteNotices(ids: readonly string[]) {
  await axios.delete(noticeEndpoints.noticesBatch, { data: { ids } });
  await refreshNoticeCollections();
  await clearNoticeDetails(ids);
}

async function refreshNoticeCollections() {
  await mutate(noticeCollectionCacheMatcher);
}

async function clearNoticeDetails(ids: readonly string[]) {
  await mutate(noticeDetailCacheMatcher(ids), undefined, { revalidate: false });
}

export function noticeCollectionCacheMatcher(key: unknown) {
  return [noticeEndpoints.notices, noticeEndpoints.top].some((endpoint) =>
    isEndpointKey(key, endpoint)
  );
}

export function noticeDetailCacheMatcher(ids: readonly string[]) {
  const endpoints = new Set(ids.map(noticeEndpoints.notice));
  return (key: unknown) => [...endpoints].some((endpoint) => isEndpointKey(key, endpoint));
}
