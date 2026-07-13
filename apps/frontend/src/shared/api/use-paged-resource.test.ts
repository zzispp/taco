import { it, expect, describe } from 'vitest';

import { pagedResourceKey, pagedResourceConfig } from './use-paged-resource';

describe('paged resource options', () => {
  it('keeps previous data by default', () => {
    expect(pagedResourceConfig()).toEqual({
      keepPreviousData: true,
      revalidateOnFocus: false,
    });
  });

  it('allows callers to disable previous data', () => {
    expect(pagedResourceConfig(false)).toEqual({
      keepPreviousData: false,
      revalidateOnFocus: false,
    });
  });

  it('builds a paged key from one options object', () => {
    expect(
      pagedResourceKey({
        endpoint: '/api/items',
        page: 1,
        pageSize: 25,
        params: { status: '0' },
      })
    ).toEqual(['/api/items', { params: { page: 2, page_size: 25, status: '0' } }]);
  });
});
