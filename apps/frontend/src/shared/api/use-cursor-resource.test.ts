import { it, expect, describe } from 'vitest';

import { cursorResourceKey, cursorResourceConfig } from './use-cursor-resource';

describe('cursor resource options', () => {
  it('keeps previous data by default so navigation does not flash empty rows', () => {
    expect(cursorResourceConfig()).toEqual({
      keepPreviousData: true,
      revalidateOnFocus: false,
    });
  });

  it('allows scope-sensitive callers to disable previous data', () => {
    expect(cursorResourceConfig(false)).toEqual({
      keepPreviousData: false,
      revalidateOnFocus: false,
    });
  });

  it('builds a cursor key without offset fields', () => {
    expect(
      cursorResourceKey({
        endpoint: '/api/items',
        request: { limit: 50, cursor: 'opaque-next' },
        params: { status: '0' },
        context: 'en',
      })
    ).toEqual(['/api/items', { params: { limit: 50, cursor: 'opaque-next', status: '0' } }, 'en']);
  });

  it('omits a missing cursor and empty filters', () => {
    expect(
      cursorResourceKey({
        endpoint: '/api/items',
        request: { limit: 20 },
        params: { status: '' },
      })
    ).toEqual(['/api/items', { params: { limit: 20 } }, '']);
  });

  it('does not allow business filters to override navigation state', () => {
    expect(
      cursorResourceKey({
        endpoint: '/api/items',
        request: { limit: 20, cursor: 'trusted-cursor' },
        params: { limit: 100, cursor: 'filter-cursor' },
      })
    ).toEqual(['/api/items', { params: { limit: 20, cursor: 'trusted-cursor' } }, '']);
  });
});
