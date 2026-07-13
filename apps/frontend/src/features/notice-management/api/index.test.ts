import { it, expect, describe } from 'vitest';

import { noticeEndpoints } from 'src/entities/notice';

import { noticeDetailCacheMatcher, noticeCollectionCacheMatcher } from './index';

describe('notice mutation cache matchers', () => {
  it('refreshes list and top notification caches', () => {
    expect(noticeCollectionCacheMatcher([noticeEndpoints.notices, { params: { page: 1 } }])).toBe(
      true
    );
    expect(noticeCollectionCacheMatcher(noticeEndpoints.top)).toBe(true);
    expect(noticeCollectionCacheMatcher(noticeEndpoints.notice('notice-1'))).toBe(false);
  });

  it('clears only details for the changed notice ids', () => {
    const matches = noticeDetailCacheMatcher(['notice-1', 'notice-2']);

    expect(matches(noticeEndpoints.notice('notice-1'))).toBe(true);
    expect(matches([noticeEndpoints.notice('notice-2'), 'cn'])).toBe(true);
    expect(matches(noticeEndpoints.notice('notice-3'))).toBe(false);
    expect(matches(noticeEndpoints.notices)).toBe(false);
  });
});
