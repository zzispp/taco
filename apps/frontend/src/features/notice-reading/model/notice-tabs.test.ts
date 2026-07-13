import type { NoticeTopItem } from 'src/entities/notice';

import { it, expect, describe } from 'vitest';

import { filterNoticeTopItems } from './notice-tabs';

const ITEMS: NoticeTopItem[] = [
  {
    notice_id: '1',
    notice_title: 'Read',
    notice_type: '1',
    create_by: 'admin',
    create_time: '2026-07-13T00:00:00Z',
    is_read: true,
  },
  {
    notice_id: '2',
    notice_title: 'Unread',
    notice_type: '2',
    create_by: 'admin',
    create_time: '2026-07-13T01:00:00Z',
    is_read: false,
  },
];

describe('filterNoticeTopItems', () => {
  it('keeps all items on the all tab', () => {
    expect(filterNoticeTopItems(ITEMS, 'all').map((item) => item.notice_id)).toEqual(['1', '2']);
  });

  it('keeps only unread items on the unread tab', () => {
    expect(filterNoticeTopItems(ITEMS, 'unread').map((item) => item.notice_id)).toEqual(['2']);
  });
});
