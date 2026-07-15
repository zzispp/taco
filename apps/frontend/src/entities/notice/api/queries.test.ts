import type { NoticeReader } from '../model/types';

import { it, expect, describe } from 'vitest';

import { cursorKey } from 'src/shared/api/pagination';

import { noticeEndpoints } from './endpoints';
import { noticeKey, noticeReaderCursorOptions, visibleNoticeReaderResource } from './queries';

describe('notice detail query key', () => {
  it.each([
    [null, true],
    ['notice-1', false],
  ] as const)('does not fetch without an id and access', (noticeId, enabled) => {
    expect(noticeKey(noticeId, enabled)).toBeNull();
  });

  it('uses the selected notice endpoint when enabled', () => {
    expect(noticeKey('notice-1', true)).toBe(noticeEndpoints.notice('notice-1'));
  });

  it('passes an opaque cursor and notice filters without offset parameters', () => {
    expect(
      cursorKey(
        noticeEndpoints.notices,
        { limit: 20, cursor: 'next-notice' },
        {
          notice_title: 'Release',
          create_by: 'admin',
          notice_type: '2',
        }
      )
    ).toEqual([
      noticeEndpoints.notices,
      {
        params: {
          limit: 20,
          cursor: 'next-notice',
          notice_title: 'Release',
          create_by: 'admin',
          notice_type: '2',
        },
      },
    ]);
  });
});

describe('notice reader resource', () => {
  const reader: NoticeReader = {
    user_id: 'user-1',
    user_name: 'admin',
    nick_name: 'Admin',
    dept_name: 'IT',
    phonenumber: null,
    read_time: '2026-07-13T00:00:00Z',
  };

  it('removes previous reader data after the current request fails', () => {
    const error = new Error('request failed');
    const resource = readerResource({ items: [reader], count: 1, error });

    expect(visibleNoticeReaderResource(resource, true)).toEqual({
      ...resource,
      data: undefined,
      items: [],
      itemCount: 0,
      nextCursor: null,
      previousCursor: null,
      hasNext: false,
      hasPrevious: false,
    });
  });

  it('disables previous data for reader requests', () => {
    expect(
      noticeReaderCursorOptions({
        noticeId: 'notice-1',
        request: { limit: 20, cursor: 'reader-next' },
        params: { search_value: 'admin' },
        enabled: true,
      })
    ).toEqual({
      endpoint: noticeEndpoints.readers('notice-1'),
      request: { limit: 20, cursor: 'reader-next' },
      params: { search_value: 'admin' },
      keepPreviousData: false,
    });
  });

  it('removes reader data while the reader query is disabled', () => {
    const resource = readerResource({ items: [reader], count: 1 });
    expect(visibleNoticeReaderResource(resource, false).items).toEqual([]);
  });
});

function readerResource(options: { items: NoticeReader[]; count: number; error?: unknown }) {
  return {
    data: undefined,
    items: options.items,
    itemCount: options.count,
    nextCursor: null,
    previousCursor: null,
    hasNext: false,
    hasPrevious: false,
    isLoading: false,
    error: options.error,
    isValidating: false,
  };
}
