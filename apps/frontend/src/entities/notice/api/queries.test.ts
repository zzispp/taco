import type { NoticeReader } from '../model/types';

import { it, expect, describe } from 'vitest';

import { pageKey } from 'src/shared/api/pagination';

import { noticeEndpoints } from './endpoints';
import { noticeKey, noticeReaderPagedOptions, visibleNoticeReaderResource } from './queries';

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

  it('converts the zero-based table page and notice filters to API query parameters', () => {
    expect(
      pageKey(noticeEndpoints.notices, 1, 20, {
        notice_title: 'Release',
        create_by: 'admin',
        notice_type: '2',
      })
    ).toEqual([
      noticeEndpoints.notices,
      {
        params: {
          page: 2,
          page_size: 20,
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
    const resource = readerResource({ items: [reader], total: 1, error });

    expect(visibleNoticeReaderResource(resource, true)).toEqual({
      ...resource,
      data: undefined,
      items: [],
      total: 0,
    });
  });

  it('disables previous data for reader requests', () => {
    expect(
      noticeReaderPagedOptions({
        noticeId: 'notice-1',
        page: 1,
        pageSize: 10,
        params: { search_value: 'admin' },
        enabled: true,
      })
    ).toEqual({
      endpoint: noticeEndpoints.readers('notice-1'),
      page: 1,
      pageSize: 10,
      params: { search_value: 'admin' },
      keepPreviousData: false,
    });
  });

  it('removes reader data while the reader query is disabled', () => {
    const resource = readerResource({ items: [reader], total: 1 });
    expect(visibleNoticeReaderResource(resource, false).items).toEqual([]);
  });
});

function readerResource(options: { items: NoticeReader[]; total: number; error?: unknown }) {
  return {
    data: undefined,
    items: options.items,
    total: options.total,
    isLoading: false,
    error: options.error,
    isValidating: false,
  };
}
