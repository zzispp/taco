import type { Notice, NoticeInput } from 'src/entities/notice';

import { NOTICE_TYPE, NOTICE_STATUS } from 'src/entities/notice';

export const EMPTY_NOTICE_INPUT: NoticeInput = {
  notice_title: '',
  notice_type: NOTICE_TYPE.NOTICE,
  notice_content: '',
  status: NOTICE_STATUS.NORMAL,
  remark: null,
};

export const NOTICE_TITLE_MAX_LENGTH = 50;

export function noticeInputFromEntity(notice?: Notice): NoticeInput {
  if (!notice) return EMPTY_NOTICE_INPUT;
  return {
    notice_title: notice.notice_title,
    notice_type: notice.notice_type,
    notice_content: notice.notice_content,
    status: notice.status,
    remark: notice.remark,
  };
}

export function normalizedNoticeInput(input: NoticeInput): NoticeInput {
  return {
    ...input,
    notice_title: input.notice_title.trim(),
    remark: input.remark?.trim() || null,
  };
}

export function noticeTitleError(input: NoticeInput): 'required' | 'tooLong' | null {
  const title = input.notice_title.trim();
  if (!title) return 'required';
  if (Array.from(title).length > NOTICE_TITLE_MAX_LENGTH) return 'tooLong';
  return null;
}
