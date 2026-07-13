import type { NoticeType, NoticeStatus } from './types';

export const NOTICE_TYPE = {
  NOTICE: '1',
  ANNOUNCEMENT: '2',
} as const satisfies Record<string, NoticeType>;

export const NOTICE_STATUS = {
  NORMAL: '0',
  CLOSED: '1',
} as const satisfies Record<string, NoticeStatus>;

export const NOTICE_PERMISSION = {
  LIST: 'system:notice:list',
  QUERY: 'system:notice:query',
  ADD: 'system:notice:add',
  EDIT: 'system:notice:edit',
  REMOVE: 'system:notice:remove',
} as const;

export const noticeTypeTranslationKeys: Record<NoticeType, string> = {
  [NOTICE_TYPE.NOTICE]: 'notice.types.notice',
  [NOTICE_TYPE.ANNOUNCEMENT]: 'notice.types.announcement',
};

export const noticeStatusTranslationKeys: Record<NoticeStatus, string> = {
  [NOTICE_STATUS.NORMAL]: 'notice.status.normal',
  [NOTICE_STATUS.CLOSED]: 'notice.status.closed',
};

export const noticeTypeColors: Record<NoticeType, 'warning' | 'success'> = {
  [NOTICE_TYPE.NOTICE]: 'warning',
  [NOTICE_TYPE.ANNOUNCEMENT]: 'success',
};

export const noticeStatusColors: Record<NoticeStatus, 'primary' | 'error'> = {
  [NOTICE_STATUS.NORMAL]: 'primary',
  [NOTICE_STATUS.CLOSED]: 'error',
};
