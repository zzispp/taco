import type { QueryParams } from 'src/shared/api/pagination';
import type { Notice, NoticeReader, NoticeSummary, NoticeTopResponse } from '../model/types';
import type { PagedResourceState, PagedResourceOptions } from 'src/shared/api/use-paged-resource';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';
import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { noticeEndpoints } from './endpoints';

export function useNotices(page: number, pageSize: number, params: QueryParams = {}) {
  return usePagedResource<NoticeSummary>({
    endpoint: noticeEndpoints.notices,
    page,
    pageSize,
    params,
  });
}

export function useNotice(noticeId: string | null, enabled: boolean) {
  return useSWR<Notice>(noticeKey(noticeId, enabled), fetcher, {
    revalidateOnFocus: false,
  });
}

export function useNoticeReaders(options: NoticeReaderQueryOptions) {
  const resourceOptions = noticeReaderPagedOptions(options);
  const resource = usePagedResource<NoticeReader>(resourceOptions);
  return visibleNoticeReaderResource(resource, resourceOptions.endpoint.length > 0);
}

export function noticeReaderPagedOptions(options: NoticeReaderQueryOptions): PagedResourceOptions {
  return {
    endpoint: options.enabled && options.noticeId ? noticeEndpoints.readers(options.noticeId) : '',
    page: options.page,
    pageSize: options.pageSize,
    params: options.params,
    keepPreviousData: false,
  };
}

export function visibleNoticeReaderResource(
  resource: PagedResourceState<NoticeReader>,
  enabled: boolean
): PagedResourceState<NoticeReader> {
  return resource.error || !enabled
    ? { ...resource, data: undefined, items: [], total: 0 }
    : resource;
}

export function useNoticeTop() {
  return useSWR<NoticeTopResponse>(noticeEndpoints.top, fetcher, {
    revalidateOnFocus: true,
  });
}

export function noticeKey(noticeId: string | null, enabled: boolean) {
  return noticeId && enabled ? noticeEndpoints.notice(noticeId) : null;
}

export type NoticeReaderQueryOptions = Readonly<{
  noticeId: string | null;
  page: number;
  pageSize: number;
  params: QueryParams;
  enabled: boolean;
}>;
