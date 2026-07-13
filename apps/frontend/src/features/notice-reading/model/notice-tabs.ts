import type { NoticeTopItem } from 'src/entities/notice';

export type NoticeTab = 'all' | 'unread';

export function filterNoticeTopItems(items: readonly NoticeTopItem[], tab: NoticeTab) {
  return tab === 'unread' ? items.filter((item) => !item.is_read) : [...items];
}
