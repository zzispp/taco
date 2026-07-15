import { it, expect } from 'vitest';

import { cursorCollectionKey, cursorCollectionShouldLoadNext } from './use-cursor-collection';

it('starts at the maximum supported batch without an offset', () => {
  expect(cursorCollectionKey({ endpoint: '/items' }, 0, null)).toEqual([
    '/items',
    { params: { limit: 100 } },
    '',
  ]);
});

it('uses only the server next cursor for subsequent batches', () => {
  expect(
    cursorCollectionKey({ endpoint: '/items', params: { status: '0' } }, 1, {
      items: [1],
      next_cursor: 'server-next',
      previous_cursor: null,
      has_next: true,
      has_previous: false,
    })
  ).toEqual(['/items', { params: { limit: 100, cursor: 'server-next', status: '0' } }, '']);
});

it('stops when the server reports no next cursor', () => {
  expect(
    cursorCollectionKey({ endpoint: '/items' }, 1, {
      items: [],
      next_cursor: null,
      previous_cursor: 'previous',
      has_next: false,
      has_previous: true,
    })
  ).toBeNull();
});

it('exposes an inconsistent server cursor instead of silently truncating the collection', () => {
  expect(() =>
    cursorCollectionKey({ endpoint: '/items' }, 1, {
      items: [1],
      next_cursor: null,
      previous_cursor: null,
      has_next: true,
      has_previous: false,
    })
  ).toThrow('Cursor collection response has_next without next_cursor');
});

it('rejects an inconsistent response before the collection silently stops loading', () => {
  expect(() =>
    cursorCollectionShouldLoadNext({
      items: [1],
      next_cursor: null,
      previous_cursor: null,
      has_next: true,
      has_previous: false,
    })
  ).toThrow('Cursor collection response has_next without next_cursor');
});
