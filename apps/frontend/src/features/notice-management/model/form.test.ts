import type { NoticeInput } from 'src/entities/notice';

import { it, expect, describe } from 'vitest';

import { noticeTitleError, normalizedNoticeInput, NOTICE_TITLE_MAX_LENGTH } from './form';

const INPUT: NoticeInput = {
  notice_title: ' Notice title ',
  notice_type: '1',
  notice_content: '# Content',
  status: '0',
  remark: ' Remark ',
};

describe('notice form', () => {
  it('normalizes title and remark without changing Markdown content', () => {
    expect(normalizedNoticeInput(INPUT)).toEqual({
      ...INPUT,
      notice_title: 'Notice title',
      notice_content: '# Content',
      remark: 'Remark',
    });
  });

  it('rejects an empty title', () => {
    expect(noticeTitleError({ ...INPUT, notice_title: '   ' })).toBe('required');
  });

  it('counts Unicode code points consistently with the backend', () => {
    const validTitle = '😀'.repeat(NOTICE_TITLE_MAX_LENGTH);
    expect(noticeTitleError({ ...INPUT, notice_title: validTitle })).toBe(null);
    expect(noticeTitleError({ ...INPUT, notice_title: `${validTitle}😀` })).toBe('tooLong');
  });
});
