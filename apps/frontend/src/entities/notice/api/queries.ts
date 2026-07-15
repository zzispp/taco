import type { QueryParams, CursorPageRequest } from 'src/shared/api/pagination';
import type { Notice, NoticeReader, NoticeSummary, NoticeTopResponse } from '../model/types';
import type {
  CursorResourceState,
  CursorResourceOptions,
} from 'src/shared/api/use-cursor-resource';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';
import { useCursorResource } from 'src/shared/api/use-cursor-resource';

import { noticeEndpoints } from './endpoints';

export function useNotices(request: CursorPageRequest, params: QueryParams = {}) {
  return useCursorResource<NoticeSummary>({
    endpoint: noticeEndpoints.notices,
    request,
    params,
  });
}

export function useNotice(noticeId: string | null, enabled: boolean) {
  return useSWR<Notice>(noticeKey(noticeId, enabled), fetcher, {
    revalidateOnFocus: false,
  });
}

export function useNoticeReaders(options: NoticeReaderQueryOptions) {
  const resourceOptions = noticeReaderCursorOptions(options);
  const resource = useCursorResource<NoticeReader>(resourceOptions);
  return visibleNoticeReaderResource(resource, resourceOptions.endpoint.length > 0);
}

export function noticeReaderCursorOptions(
  options: NoticeReaderQueryOptions
): CursorResourceOptions {
  return {
    endpoint: options.enabled && options.noticeId ? noticeEndpoints.readers(options.noticeId) : '',
    request: options.request,
    params: options.params,
    keepPreviousData: false,
  };
}

export function visibleNoticeReaderResource(
  resource: CursorResourceState<NoticeReader>,
  enabled: boolean
): CursorResourceState<NoticeReader> {
  return resource.error || !enabled
    ? {
        ...resource,
        data: undefined,
        items: [],
        itemCount: 0,
        nextCursor: null,
        previousCursor: null,
        hasNext: false,
        hasPrevious: false,
      }
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
  request: CursorPageRequest;
  params: QueryParams;
  enabled: boolean;
}>;
